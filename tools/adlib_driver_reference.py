#!/usr/bin/env python3
"""Optional x86-16 oracle. Execute the verified original driver, not a translation.

Requires isolated Unicorn 2.1.4. No DOS, PIT, OPL synthesis or wall-time emulation.
Only driver calls, RAM records and OUT 388/389 ordering are compared. DX=0 ABI.
"""
import hashlib
import struct

class Driver:
    def __init__(self, data, *, verify=True):
        if verify and hashlib.sha256(data).hexdigest()!='ad4f33eb383c60b5a67945987f27580fc0540e38558d4c158efe1a672db5f490':
            raise ValueError('original driver hash mismatch')
        import unicorn
        from unicorn import x86_const as x
        if unicorn.__version__!='2.1.4':raise ValueError('Unicorn 2.1.4 required')
        self.x=x;self.u=unicorn.Uc(unicorn.UC_ARCH_X86,unicorn.UC_MODE_16)
        self.u.mem_map(0,0x100000);self.u.mem_write(0x10000,bytes(data))
        self.writes=[];self.registers=[0]*256;self.address=0;self.step=0
        self.opcodes={};self.loops=0;self.stops=0;self.channel_loops=[0]*9
        self.u.hook_add(unicorn.UC_HOOK_INSN,self.output,None,1,0,x.UC_X86_INS_OUT)
        self.u.hook_add(unicorn.UC_HOOK_INSN,lambda *args:0,None,1,0,x.UC_X86_INS_IN)
        # Count only command fetch blocks, not every instruction.
        self.u.hook_add(unicorn.UC_HOOK_CODE,self.command,None,0x1019e,0x1019e)
        self.u.hook_add(unicorn.UC_HOOK_CODE,self.loop,None,0x103fa,0x103fa)
    def command(self,u,address,size,user):
        command=self.u.mem_read(0x10000+self.u.reg_read(self.x.UC_X86_REG_BX),1)[0]
        self.opcodes[command]=self.opcodes.get(command,0)+1
        if command in (0x83,0x85):self.stops+=1
    def loop(self,*args):
        self.loops+=1;self.channel_loops[(self.u.reg_read(self.x.UC_X86_REG_DI)-1452)//20]+=1
    def output(self,u,port,size,value,user):
        if size!=1:raise ValueError('non-byte OPL output')
        if port==0x388:self.address=value
        elif port==0x389:self.writes.append([self.address,value]);self.registers[self.address]=value
        else:raise ValueError('unexpected port '+hex(port))
    def call(self,ax):
        x=self.x;u=self.u
        for r,v in [(x.UC_X86_REG_AX,ax),(x.UC_X86_REG_BX,0),(x.UC_X86_REG_CX,0),(x.UC_X86_REG_DX,0),(x.UC_X86_REG_SI,0),(x.UC_X86_REG_DI,0),(x.UC_X86_REG_BP,0),(x.UC_X86_REG_ES,0),(x.UC_X86_REG_CS,0x1000),(x.UC_X86_REG_DS,0),(x.UC_X86_REG_SS,0x9000),(x.UC_X86_REG_SP,0xff00),(x.UC_X86_REG_EFLAGS,2)]:u.reg_write(r,v)
        u.mem_write(0x9ff00,struct.pack('<HHH',0,0x8000,2));u.emu_start(0x10000,0x80000,count=1000000)
        if u.reg_read(x.UC_X86_REG_CS)!=0x8000 or u.reg_read(x.UC_X86_REG_IP)!=0:raise ValueError('driver instruction budget exceeded')
    def request(self,command):
        if command=='n':return
        if command=='s':
            self.call(0x400);self.call(0x200);return
        index=int(command[1:]);limit=21 if command[0]=='m' else 18
        if command[0] not in 'me' or not 0<=index<limit:raise ValueError('invalid action')
        if command[0]=='m':self.call(0x500);self.call(0x300+index+1)
        else:self.call(0x400+index+1)
    def read(self):
        self.call(0);u=self.u
        result=dict(step=self.step,channels=[list(u.mem_read(0x105ac+i*20,20)) for i in range(9)],globals=[int.from_bytes(u.mem_read(0x105a8,2),'little'),int.from_bytes(u.mem_read(0x10b68,2),'little'),u.mem_read(0x10b6a,1)[0],u.mem_read(0x10b6b,1)[0],int.from_bytes(u.mem_read(0x10b6c,2),'little')],flags=list(u.mem_read(0x100ed,4)),registers=self.registers.copy(),writes=self.writes.copy())
        self.step+=1;self.writes.clear();return result

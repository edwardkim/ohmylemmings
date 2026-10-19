{ Own synthetic inputs. Selection/assignment method bodies are extracted unchanged. }
procedure PointerProbe(G:TLemmingGame);
const Actions:array[0..17] of TLemmingAction=(TLemmingAction.Walking,TLemmingAction.Jumping,
  TLemmingAction.Falling,TLemmingAction.Splatting,TLemmingAction.Digging,TLemmingAction.Bashing,
  TLemmingAction.Mining,TLemmingAction.Building,TLemmingAction.Shrugging,TLemmingAction.Blocking,
  TLemmingAction.Ohnoing,TLemmingAction.Exploding,TLemmingAction.Climbing,TLemmingAction.Hoisting,
  TLemmingAction.Floating,TLemmingAction.Exiting,TLemmingAction.Drowning,TLemmingAction.Vaporizing);
var ai,bi,x,y,r,n,k,v,sk:Integer;L,W,A,B,Rt:TLemming; H:QWord;
  function Id(L:TLemming):Integer;begin Result:=-1;if L<>nil then Result:=L.ListIndex end;
  procedure State(L:TLemming;act:TLemmingAction;index,px:Integer);
  begin
    L.XPos:=px;L.YPos:=100;L.xDelta:=1;G.Transition(L,act);L.YPos:=100;
    { Metadata copied from the pinned Styles.Base table, verified by builder. }
    L.FrameLeftDx:=-8;L.FrameTopDy:=-10;
    case act of
      TLemmingAction.Building,TLemmingAction.Mining,TLemmingAction.Exiting:L.FrameTopDy:=-13;
      TLemmingAction.Digging,TLemmingAction.Climbing,TLemmingAction.Hoisting:L.FrameTopDy:=-12;
      TLemmingAction.Floating:L.FrameTopDy:=-16;
      TLemmingAction.Vaporizing:L.FrameTopDy:=-14;
      TLemmingAction.Exploding:begin L.FrameTopDy:=-25;L.FrameLeftDx:=-16 end;
    end;
    L.ListIndex:=index;L.IsRemoved:=False;L.ObjectBelow:=128;L.ObjectInFront:=128;
  end;
  procedure Hit;
  begin
    G.CursorPoint:=Point(x-3,y+2);G.RightMouseButtonHeldDown:=r=1;
    n:=G.PrioritizedHitTest(A,B,G.CursorPoint,True);
    WriteLn('H ',k,' ',n,' ',Id(A),' ',Id(B));Inc(k);
  end;
begin
  L:=TLemming.Create;W:=TLemming.Create;SetLength(G.fLemmingList.Items,1);G.fLemmingList.Items[0]:=L;k:=0;
  for ai:=0 to 17 do begin
    State(L,Actions[ai],0,100);
    for x:=80 to 112 do for y:=70 to 106 do for r:=0 to 1 do Hit;
  end;
  SetLength(G.fLemmingList.Items,2);G.fLemmingList.Items[1]:=W;
  for ai:=0 to 17 do for bi:=0 to 17 do for r:=0 to 1 do begin
    State(L,Actions[ai],0,100);State(W,Actions[bi],1,105);x:=100;y:=90;Hit;
  end;
  G.SkillMethods[TLemmingAction.Blocking]:=G.AssignBlocker;
  SkillPanelButtonToAction[TSkillPanelButton.Blocker]:=TLemmingAction.Blocking;
  G.fSelectedSkill:=TSkillPanelButton.Blocker;Include(G.fMechanics,TMechanic.RightClickGlitch);
  for v:=0 to 1 do begin
    for n:=0 to Length(G.fObjectMap.Cells)-1 do G.fObjectMap.Cells[n]:=128;
    G.fLastNonPrioritizedLemming:=nil;G.CurrBlockerCount:=2;
    State(L,TLemmingAction.Walking,0,32);State(W,TLemmingAction.Building,1,200);
    L.ObjectBelow:=0;G.CursorPoint:=Point(32-3,95+2);G.RightMouseButtonHeldDown:=False;
    G.PrioritizedHitTest(A,B,G.CursorPoint,True);
    if v=1 then L.IsRemoved:=True;
    G.CursorPoint:=Point(200-3,95+2);G.RightMouseButtonHeldDown:=True;
    Rt:=G.ProcessSkillAssignment(False);H:=$cbf29ce484222325;
    for n:=0 to Length(G.fObjectMap.Cells)-1 do H:=(H xor G.fObjectMap.Cells[n])*QWord($100000001b3);
    WriteLn('R ',v,' ',Id(Rt),' ',G.CurrBlockerCount,' ',LowerCase(BoolToStr(L.IsRemoved,True)),' ',LowerCase(IntToHex(H,16)));
  end;
  k:=0;
  for ai:=0 to 17 do for bi:=0 to 17 do for sk:=0 to 1 do for v:=0 to 5 do begin
    State(L,Actions[ai],0,32);State(W,Actions[bi],1,60);
    if v=1 then L.ObjectBelow:=137;
    if v=2 then W.ObjectBelow:=137;
    if v=3 then L.YPos:=-L.FrameTopDy-6;
    if v=4 then W.YPos:=-W.FrameTopDy-6;
    n:=2;if v=5 then n:=0;
    G.CurrBuilderCount:=n;G.CurrDiggerCount:=n;
    if sk=0 then begin Rt:=G.AssignBuilder(L,W);n:=G.CurrBuilderCount end
    else begin Rt:=G.AssignDigger(L,W);n:=G.CurrDiggerCount end;
    WriteLn('F ',k,' ',Id(Rt),' ',n);Inc(k);
  end;

end;

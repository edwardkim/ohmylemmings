{ Own synthetic driver; all Assign* and Transition bodies come from reference.inc. }
function ActionName(A:TLemmingAction):String;
begin
  Result:=GetEnumName(TypeInfo(TLemmingAction),Ord(A));
  if A=TLemmingAction.Ohnoing then Result:='OhNo';
end;

procedure AssignmentProbe(G:TLemmingGame);
const
  Actions:array[0..14] of TLemmingAction=(TLemmingAction.Walking,TLemmingAction.Jumping,
    TLemmingAction.Falling,TLemmingAction.Splatting,TLemmingAction.Digging,
    TLemmingAction.Bashing,TLemmingAction.Mining,TLemmingAction.Building,
    TLemmingAction.Shrugging,TLemmingAction.Blocking,TLemmingAction.Ohnoing,
    TLemmingAction.Exploding,TLemmingAction.Climbing,TLemmingAction.Hoisting,TLemmingAction.Floating);
  Skills:array[0..7] of String=('Climber','Floater','Bomber','Blocker','Builder','Basher','Miner','Digger');
  Variants:array[0..10] of String=('normal','empty','athlete','armed','steel_below',
    'steel_front','oneway_left','oneway_right','overlap','head_above','head_edge');
var ai,skill,d,v,i,n,fallen,bricks:Integer; L,R:TLemming; H:QWord; a:TLemmingAction;
begin
  for ai:=0 to High(Actions) do for skill:=0 to 7 do
    for d:=-1 to 1 do if d<>0 then for v:=0 to High(Variants) do begin
      for i:=0 to Length(G.fObjectMap.Cells)-1 do G.fObjectMap.Cells[i]:=128;
      a:=Actions[ai]; L:=TLemming.Create; L.XPos:=32; L.YPos:=100; L.xDelta:=d;
      G.Transition(L,a);
      { Mining increments Y at assignment; these are already-current action states. }
      L.YPos:=100; L.Frame:=3; if L.MaxFrame=0 then L.Frame:=0;
      L.EndOfAnimation:=False; L.IsNewDigger:=False;
      if a=TLemmingAction.Falling then L.Fallen:=12;
      L.IsClimber:=(v=2) or (a in [TLemmingAction.Climbing,TLemmingAction.Hoisting]);
      L.IsFloater:=(v=2) or (a=TLemmingAction.Floating);
      L.FloatParametersTableIndex:=0;if L.IsFloater then L.FloatParametersTableIndex:=7;
      if (v=3) and (a<>TLemmingAction.Splatting) then L.ExplosionTimer:=42;
      L.ObjectBelow:=128; L.ObjectInFront:=128;
      if v=4 then L.ObjectBelow:=137;
      case v of 5:L.ObjectInFront:=137;6:L.ObjectInFront:=135;7:L.ObjectInFront:=136 end;
      if v=9 then L.YPos:=-L.FrameTopDy-6;
      if v=10 then L.YPos:=-L.FrameTopDy-5;
      if v=8 then G.WriteObjectMap(32,L.YPos,131);
      G.CurrClimberCount:=1;G.CurrFloaterCount:=1;G.CurrBomberCount:=1;G.CurrBlockerCount:=1;
      G.CurrBuilderCount:=1;G.CurrBasherCount:=1;G.CurrMinerCount:=1;G.CurrDiggerCount:=1;
      if v=1 then case skill of 0:G.CurrClimberCount:=0;1:G.CurrFloaterCount:=0;
        2:G.CurrBomberCount:=0;3:G.CurrBlockerCount:=0;4:G.CurrBuilderCount:=0;
        5:G.CurrBasherCount:=0;6:G.CurrMinerCount:=0;7:G.CurrDiggerCount:=0 end;
      case skill of
        0:begin R:=G.AssignClimber(L,nil);n:=G.CurrClimberCount end;
        1:begin R:=G.AssignFloater(L,nil);n:=G.CurrFloaterCount end;
        2:begin R:=G.AssignBomber(L,nil);n:=G.CurrBomberCount end;
        3:begin R:=G.AssignBlocker(L,nil);n:=G.CurrBlockerCount end;
        4:begin R:=G.AssignBuilder(L,nil);n:=G.CurrBuilderCount end;
        5:begin R:=G.AssignBasher(L,nil);n:=G.CurrBasherCount end;
        6:begin R:=G.AssignMiner(L,nil);n:=G.CurrMinerCount end;
        7:begin R:=G.AssignDigger(L,nil);n:=G.CurrDiggerCount end;
      end;
      { Fields ignored by the current action are normalized, as in existing traces. }
      fallen:=0;if L.Action=TLemmingAction.Falling then fallen:=L.Fallen;
      bricks:=0;if L.Action=TLemmingAction.Building then bricks:=L.NumberOfBricksLeft;
      H:=$cbf29ce484222325;
      for i:=0 to Length(G.fObjectMap.Cells)-1 do H:=(H xor G.fObjectMap.Cells[i])*QWord($100000001b3);
      WriteLn('{"initial":"',ActionName(a),'","skill":"',Skills[skill],
        '","direction":',d,',"variant":"',Variants[v],'","accepted":',B(R<>nil),
        ',"remaining":',n,',"action":"',ActionName(L.Action),'","y":',L.YPos,
        ',"frame":',L.Frame,',"end":',B(L.EndOfAnimation),',"fallen":',fallen,
        ',"new_digger":',B((L.Action=TLemmingAction.Digging) and L.IsNewDigger),
        ',"bricks":',bricks,',"climber":',B(L.IsClimber),',"floater":',B(L.IsFloater),
        ',"float_index":',L.FloatParametersTableIndex,',"walking_once":',
        B((L.Action=TLemmingAction.Walking) and (L.AnimationType=TLemmingAnimationType.Once)),
        ',"timer":',L.ExplosionTimer,',"map_hash":"',LowerCase(IntToHex(H,16)),'"}');
      L.Free;
    end;
end;

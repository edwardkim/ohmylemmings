{ G05 adapter declarations and driver. Reference method bodies are generated
  from the pinned local source into reference.inc and never checked in here. }
program G05Reference;
{$mode delphi}{$scopedenums on}{$B-}
uses SysUtils, Classes;
const
  ALPHA_TERRAIN = 1; LEMMING_MIN_X = 0; LEMMING_MAX_X = 1647;
  LEMMING_MAX_Y = 163; HEAD_MIN_Y = -5; MAX_FALLDISTANCE = 60;
  COMBINE_FLAG_BUILDER = 1; ACTION_BIT_FLOATING = 1 shl 12;
  ACTION_BIT_DIGGING = 1 shl 7; DOM_NONE = 0; MUSIC_INDEX = 0;
type
  TLemmingAction = (None, Walking, Jumping, Falling, Splatting, Blocking,
    Exiting, Digging, Building, Ohnoing, Exploding, Floating, Mining, Climbing);
  TLemmingAnimationType = (Loop, Once);
  TMechanic = (PauseGlitch, FallerStartsWith3, EntranceX25);
  TMechanics = set of TMechanic;
  TSoundOption = (Music); TSoundOptions = set of TSoundOption;
  TPauseCommandMode = (None, PauseKey);
  TMeta = record FrameCount, FootX, FootY: Integer; AnimationType: TLemmingAnimationType end;
  TLemmingAnimationSet = class
    class var AnimationIndices: array[TLemmingAction, Boolean] of Integer;
    var MetaLemmingAnimationList: array[0..13] of TMeta;
        LemmingBitmaps: array[0..13] of Integer;
  end;
  TStyle = class LemmingAnimationSet: TLemmingAnimationSet end;
  TLemming = class
    XPos,YPos,xDelta,Frame,MaxFrame,FrameTopDy,FrameLeftDx,Fallen: Integer;
    NumberOfBricksLeft,ExplosionTimer,FloatParametersTableIndex,ListIndex,Born: Integer;
    PixelCombine,LAB,ObjectInFront,ObjectBelow,ActionBits: Integer;
    Action: TLemmingAction; AnimationType: TLemmingAnimationType;
    LMA: TMeta; CombineFlags: Byte;
    EndOfAnimation,IsBlocking,IsNewDigger,IsClimber,IsFloater,IsRemoved: Boolean;
    constructor Create;
    function GetRTL: Boolean;
    function ActionIn(flags: Integer): Boolean;
    property RTL: Boolean read GetRTL;
  end;
  TLemmingMethod = function(L: TLemming): Boolean of object;
  TWorld = class
    Width,Height: Integer; Cells: array of Byte;
    function GetPixel(X,Y:Integer): Integer;
    property Pixel[X,Y:Integer]:Integer read GetPixel; default;
  end;
  TInteractiveObjectInfo = class
    Obj: record Left,Top:Integer end;
    Triggered:Boolean; CurrentFrame:Integer;
  end;
  TEntranceEnumerator = class
    Items: array of TInteractiveObjectInfo; Index:Integer;
    function MoveNext:Boolean;
    function GetCurrent:TInteractiveObjectInfo;
    property Current:TInteractiveObjectInfo read GetCurrent;
  end;
  TEntrances = class
    Items: array of TInteractiveObjectInfo; Count: Integer;
    function GetItem(i:Integer):TInteractiveObjectInfo;
    function GetEnumerator:TEntranceEnumerator;
    property Item[i:Integer]:TInteractiveObjectInfo read GetItem; default;
  end;
  TLemmingList = class
    Items: array of TLemming;
    function Add(L:TLemming):Integer;
  end;
  TToolbar = class
    procedure SetInfoMinutes(v:Integer); procedure SetInfoSeconds(v:Integer);
    procedure SetPauseHighlight(v:Boolean);
  end;
  TSoundManager = class
    MusicIsPlaying:array[0..0] of Boolean;
    procedure PlayMusic(v:Integer);
  end;
  TRecorder = class fRecordedGlitchPauseIterations:Integer end;
  TLemmingGame = class
    fWorld:TWorld; fStyle:TStyle; fLemmingList:TLemmingList;
    fMechanics:TMechanics; fToolbar:TToolbar; fSoundMgr:TSoundManager;
    fRecorder:TRecorder; Entrances:TEntrances;
    LemmingMethods:array[TLemmingAction] of TLemmingMethod;
    DosEntranceOrderTable:array[0..3] of Integer;
    fHandleLemmingCalls,fCurrentIteration,fGlitchPauseIterations,fClockFrame:Integer;
    fParticleFinishTimer,Minutes,Seconds,NextLemmingCountdown:Integer;
    currReleaseRate,MaxNumLemmings,LemmingsReleased,LemmingsOut,LemmingsRemoved:Integer;
    fIsNukedByUser,EntrancesOpened,fStartupMusicAfterEntrance,fIsPaused: Boolean;
    fIsPausedExt,FastForward,fReplaying:Boolean; fSoundOpts:TSoundOptions;
    Events:String;
    property Mechanics:TMechanics read fMechanics;
    property CurrentIteration:Integer read fCurrentIteration;
    property IsPaused:Boolean read fIsPaused;
    property ToolBar:TToolbar read fToolbar;
    function HasPixelAt(X,Y:Integer):Boolean;
    function HasPixelAt_ClipY(X,Y,minY:Integer):Boolean;
    procedure Transition(L:TLemming; aAction:TLemmingAction; DoTurn:Boolean=False);
    procedure TurnAround(L:TLemming);
    function HandleLemming(L:TLemming):Boolean;
    function HandleWalking(L:TLemming):Boolean;
    function HandleJumping(L:TLemming):Boolean;
    function HandleFalling(L:TLemming):Boolean;
    function HandleSplatting(L:TLemming):Boolean;
    function CheckForLevelTopBoundary(L:TLemming; LocalFrameTopDy:Integer=0):Boolean;
    procedure RemoveLemming(L:TLemming);
    procedure IncrementIteration;
    procedure CheckSpawnLemming;
    procedure BtnPause(mode:TPauseCommandMode=TPauseCommandMode.None);
    procedure BtnPauseStop;
    procedure CueSoundEffect(v:Integer);
    procedure SaveMap(L:TLemming); procedure SetBlockerField(L:TLemming);
    procedure UpdatePixelCombine(L:TLemming); procedure RegainControl;
    procedure RecordStartPause; procedure RecordEndPause;
    procedure Event(const s:String);
  end;
var
  SoundData:record SFX_SPLAT,SFX_YIPPEE,SFX_OHNO,SFX_EXPLOSION,
    SFX_SILENTDEATH,SFX_LETSGO,SFX_ENTRANCE:Integer end;
  GameOptions:record HighlightedPauseButton:Boolean end;
const CombineLemming=0;

constructor TLemming.Create;
begin inherited; Action:=TLemmingAction.None; xDelta:=1 end;
function TLemming.GetRTL:Boolean; begin Result:=xDelta<0 end;
function TLemming.ActionIn(flags:Integer):Boolean;
begin Result:=((Action=TLemmingAction.Floating) and (flags and ACTION_BIT_FLOATING<>0)) or
 ((Action=TLemmingAction.Digging) and (flags and ACTION_BIT_DIGGING<>0)) end;
function TWorld.GetPixel(X,Y:Integer):Integer;
begin if (X<0) or (Y<0) or (X>=Width) or (Y>=Height) then raise Exception.Create('unclipped pixel'); Result:=Cells[Y*Width+X] end;
function TEntranceEnumerator.MoveNext:Boolean; begin Inc(Index); Result:=Index<Length(Items) end;
function TEntranceEnumerator.GetCurrent:TInteractiveObjectInfo; begin Result:=Items[Index] end;
function TEntrances.GetItem(i:Integer):TInteractiveObjectInfo; begin Result:=Items[i] end;
function TEntrances.GetEnumerator:TEntranceEnumerator;
begin Result:=TEntranceEnumerator.Create; Result.Items:=Items; Result.Index:=-1 end;
function TLemmingList.Add(L:TLemming):Integer;
begin Result:=Length(Items); SetLength(Items,Result+1); Items[Result]:=L end;
procedure TToolbar.SetInfoMinutes(v:Integer); begin end;
procedure TToolbar.SetInfoSeconds(v:Integer); begin end;
procedure TToolbar.SetPauseHighlight(v:Boolean); begin end;
procedure TSoundManager.PlayMusic(v:Integer); begin raise Exception.Create('music unsupported') end;
procedure TLemmingGame.SaveMap(L:TLemming); begin raise Exception.Create('blockers unsupported') end;
procedure TLemmingGame.SetBlockerField(L:TLemming); begin raise Exception.Create('blockers unsupported') end;
procedure TLemmingGame.UpdatePixelCombine(L:TLemming); begin raise Exception.Create('pixel combining unsupported') end;
procedure TLemmingGame.RegainControl; begin raise Exception.Create('reference replay recorder unsupported') end;
procedure TLemmingGame.RecordStartPause; begin end;
procedure TLemmingGame.RecordEndPause; begin end;
procedure TLemmingGame.Event(const s:String);
begin if Events<>'' then Events:=Events+','; Events:=Events+'"'+s+'"' end;
procedure TLemmingGame.CueSoundEffect(v:Integer);
begin if v=SoundData.SFX_LETSGO then Event('LetsGo')
 else if v=SoundData.SFX_ENTRANCE then Event('EntranceSound') end;

{$I reference.inc}

function B(v:Boolean):String; begin if v then Result:='true' else Result:='false' end;
procedure Snapshot(G:TLemmingGame; step:Integer);
var i:Integer; L:TLemming; removal:String;
begin
  Write('{"control_step":',step,',"iteration":',G.CurrentIteration,',"paused":',B(G.IsPaused),
    ',"seconds_remaining":',G.Minutes*60+G.Seconds,',"clock_frame":',G.fClockFrame,
    ',"glitch_pause_iterations":',G.fGlitchPauseIterations,',"release_rate":',G.currReleaseRate,
    ',"spawn_countdown":',G.NextLemmingCountdown,',"entrances_open":',B(G.EntrancesOpened),
    ',"released":',G.LemmingsReleased,',"alive":',G.LemmingsOut,',"removed":',G.LemmingsRemoved,',"actors":[');
  for i:=0 to Length(G.fLemmingList.Items)-1 do begin
    if i>0 then Write(','); L:=G.fLemmingList.Items[i];
    removal:='null';
    if L.IsRemoved then if L.Action=TLemmingAction.Splatting then removal:='"SplattingFinished"' else removal:='"BottomBoundary"';
    Write('{"id":',L.ListIndex,',"born_iteration":',L.Born,',"x":',L.XPos,',"y":',L.YPos,
      ',"direction":',L.xDelta,',"action":"');
    case L.Action of TLemmingAction.Walking:Write('Walking'); TLemmingAction.Jumping:Write('Jumping');
      TLemmingAction.Falling:Write('Falling'); TLemmingAction.Splatting:Write('Splatting');
      else raise Exception.Create('unsupported action') end;
    Write('","frame":',L.Frame,',"end_of_animation":',B(L.EndOfAnimation),',"fallen":',L.Fallen,
      ',"removed":',removal,'}');
  end;
  WriteLn('],"events":[',G.Events,']}');
end;
var
  G:TLemmingGame; F:TFileStream; steps,count,pg,ft,i,j,step,sequence,code,rate:Integer;
  oldReleased,oldIteration,initialRate:Integer; accepted:Boolean; a:TLemmingAction;
  Inputs:array of record step,sequence,code,rate:Integer end;
begin
  try
    if ParamCount<>1 then raise Exception.Create('usage: reference MASK; normalized script on stdin');
    G:=TLemmingGame.Create; G.fWorld:=TWorld.Create; G.Entrances:=TEntrances.Create;
    G.fLemmingList:=TLemmingList.Create; G.fRecorder:=TRecorder.Create;
    ReadLn(G.fWorld.Width,G.fWorld.Height,G.Minutes,G.MaxNumLemmings,initialRate,count,pg,ft,steps);
    G.currReleaseRate:=initialRate; G.NextLemmingCountdown:=20;
    if pg=1 then Include(G.fMechanics,TMechanic.PauseGlitch);
    if ft=1 then Include(G.fMechanics,TMechanic.FallerStartsWith3);
    G.Entrances.Count:=count; SetLength(G.Entrances.Items,count);
    for i:=0 to count-1 do begin
      G.Entrances.Items[i]:=TInteractiveObjectInfo.Create;
      ReadLn(G.Entrances.Items[i].Obj.Left,G.Entrances.Items[i].Obj.Top);
    end;
    case count of
      2:begin G.DosEntranceOrderTable[0]:=0; G.DosEntranceOrderTable[1]:=1; G.DosEntranceOrderTable[2]:=1; G.DosEntranceOrderTable[3]:=0 end;
      3:begin G.DosEntranceOrderTable[0]:=0; G.DosEntranceOrderTable[1]:=1; G.DosEntranceOrderTable[2]:=2; G.DosEntranceOrderTable[3]:=1 end;
      4:for i:=0 to 3 do G.DosEntranceOrderTable[i]:=i;
    end;
    ReadLn(count); SetLength(Inputs,count);
    for i:=0 to count-1 do ReadLn(Inputs[i].step,Inputs[i].sequence,Inputs[i].code,Inputs[i].rate);
    F:=TFileStream.Create(ParamStr(1),fmOpenRead);
    if F.Size<>G.fWorld.Width*G.fWorld.Height then raise Exception.Create('mask size');
    SetLength(G.fWorld.Cells,F.Size); F.ReadBuffer(G.fWorld.Cells[0],F.Size); F.Free;
    G.fStyle:=TStyle.Create; G.fStyle.LemmingAnimationSet:=TLemmingAnimationSet.Create;
    for a:=Low(TLemmingAction) to High(TLemmingAction) do begin
      TLemmingAnimationSet.AnimationIndices[a,False]:=Ord(a);
      TLemmingAnimationSet.AnimationIndices[a,True]:=Ord(a);
      with G.fStyle.LemmingAnimationSet.MetaLemmingAnimationList[Ord(a)] do begin
        FrameCount:=1; FootX:=8; FootY:=10; AnimationType:=TLemmingAnimationType.Once;
        case a of
          TLemmingAction.Walking:begin FrameCount:=8; AnimationType:=TLemmingAnimationType.Loop end;
          TLemmingAction.Falling:begin FrameCount:=4; AnimationType:=TLemmingAnimationType.Loop end;
          TLemmingAction.Splatting:FrameCount:=16;
        end;
      end;
    end;
    G.LemmingMethods[TLemmingAction.Walking]:=G.HandleWalking;
    G.LemmingMethods[TLemmingAction.Jumping]:=G.HandleJumping;
    G.LemmingMethods[TLemmingAction.Falling]:=G.HandleFalling;
    G.LemmingMethods[TLemmingAction.Splatting]:=G.HandleSplatting;
    SoundData.SFX_LETSGO:=1; SoundData.SFX_ENTRANCE:=2;
    Snapshot(G,0); j:=0;
    for step:=0 to steps-1 do begin
      G.Events:='';
      while (j<Length(Inputs)) and (Inputs[j].step=step) do begin
        sequence:=Inputs[j].sequence; code:=Inputs[j].code; rate:=Inputs[j].rate; accepted:=True;
        case code of
          0:G.BtnPause;
          1:G.BtnPauseStop;
          2:begin accepted:=(rate>=initialRate) and (rate<=99); if accepted then G.currReleaseRate:=rate end;
          else raise Exception.Create('unsupported input');
        end;
        G.Event('Input:'+IntToStr(step)+':'+IntToStr(sequence)+':'+B(accepted)); Inc(j);
      end;
      if not G.IsPaused then begin
        oldIteration:=G.CurrentIteration; G.IncrementIteration;
        if (oldIteration=34) then G.Event('EntrancesOpened');
        oldReleased:=G.LemmingsReleased; G.CheckSpawnLemming;
        if G.LemmingsReleased<>oldReleased then G.Event('Spawn:'+IntToStr(oldReleased)+':'+
          IntToStr(G.DosEntranceOrderTable[oldReleased mod 4])+':'+IntToStr(G.CurrentIteration));
        for i:=0 to Length(G.fLemmingList.Items)-1 do
          if not G.fLemmingList.Items[i].IsRemoved then G.HandleLemming(G.fLemmingList.Items[i]);
      end;
      Snapshot(G,step+1);
    end;
  except on E:Exception do begin WriteLn(StdErr,E.Message); Halt(2) end end;
end.

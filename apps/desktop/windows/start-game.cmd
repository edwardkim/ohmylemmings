@echo off
setlocal
pushd "%~dp0"
if errorlevel 1 goto directory_error
set "lemmings_log_dir=%LOCALAPPDATA%\OhMyLemmings"
if not defined LOCALAPPDATA set "lemmings_log_dir=%TEMP%\OhMyLemmings"
if not exist "%lemmings_log_dir%" mkdir "%lemmings_log_dir%"
set "lemmings_log=%lemmings_log_dir%\last-run.log"
>"%lemmings_log%" echo Starting Oh My Lemmings
if errorlevel 1 goto log_error
if not exist "OhMyLemmings.exe" goto missing_exe
if not exist "SDL2.dll" goto missing_dll
"OhMyLemmings.exe" %* >>"%lemmings_log%" 2>&1
set "lemmings_exit=%errorlevel%"
if not "%lemmings_exit%"=="0" goto game_error
popd
exit /b 0
:missing_exe
>>"%lemmings_log%" echo OhMyLemmings.exe is missing. Extract the complete ZIP first.
set "lemmings_exit=1"
goto game_error
:missing_dll
>>"%lemmings_log%" echo SDL2.dll is missing. Extract the complete ZIP first.
set "lemmings_exit=1"
:game_error
echo.
echo Game did not start or exited with an error.
type "%lemmings_log%"
echo.
echo Log: "%lemmings_log%"
pause
popd
exit /b %lemmings_exit%
:log_error
echo Could not write the log in "%lemmings_log_dir%".
pause
popd
exit /b 1
:directory_error
echo Could not open the game directory. Extract the complete ZIP first.
pause
exit /b 1

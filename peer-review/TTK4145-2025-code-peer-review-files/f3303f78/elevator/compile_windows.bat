@echo off
setlocal

set "RELEASE=%1"
if "%RELEASE%"=="" set "RELEASE=win_debug"

REM Run tests
set "MIX_ENV=test"
echo Running tests...
for /f "delims=" %%i in ('mix test') do set "TEST_OUTPUT=%%i"

echo %TEST_OUTPUT% | find "0 failures" >nul
if errorlevel 1 (
    echo Tests failed. Aborting compilation.
    exit /b 1
) else (
    echo Tests passed. Proceeding with compilation...
)

REM Compile the Mix application
echo Compiling Mix application for release: %RELEASE%
set "MIX_ENV=prod"
call mix release %RELEASE% --overwrite --quiet
copy "rel\start_elevator.bat" "releases\%RELEASE%\start_elevator.bat" >nul

endlocal

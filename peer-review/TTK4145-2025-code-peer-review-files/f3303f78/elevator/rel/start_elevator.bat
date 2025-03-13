@echo off
setlocal

set "SCRIPT_DIR=%~dp0"
cd /d "%SCRIPT_DIR%/bin"

for %%f in (*.bat) do (
    set "BAT_FILE=%%f"
    goto :found
)

:found
if defined BAT_FILE (
	start "" %BAT_FILE% start
) else (
    echo No .bat file found in the bin directory.
)

endlocal

@echo off
setlocal enabledelayedexpansion
title OpenSeason Document Builder

cd /d "%~dp0"

if "%~1"=="exe" (
    call :run_exe
    exit /b %ERRORLEVEL%
)
if "%~1"=="run" (
    call :run_exe
    exit /b %ERRORLEVEL%
)
if "%~1"=="dev" (
    call :run_dev
    exit /b %ERRORLEVEL%
)
if "%~1"=="test" (
    call :run_test
    exit /b %ERRORLEVEL%
)
if "%~1"=="build" (
    call :run_build
    exit /b %ERRORLEVEL%
)
if "%~1"=="cases" (
    call :open_cases
    exit /b %ERRORLEVEL%
)
if "%~1"=="packs" (
    call :open_packs
    exit /b %ERRORLEVEL%
)

:menu
cls
echo =======================================================
echo          OpenSeason - Document Builder Launcher
echo =======================================================
echo.
echo   [1] Launch Standalone App (OpenSeason.exe - Final Version)
echo   [2] Launch Dev Mode Window (Vite dev server + hot reload)
echo   [3] Run Agent E2E Test Suite (verify PDF and checks)
echo   [4] Rebuild Release Executable (OpenSeason.exe)
echo   [5] Open Cases Folder (Documents\JustLegal\Cases)
echo   [6] Open Packs Folder (Documents\JustLegal\Packs)
echo   [7] Exit
echo.
set /p choice="Select an option [1-7] (Default is 1): "

if "%choice%"=="" set choice=1
if "%choice%"=="1" (
    call :run_exe
    goto :menu
)
if "%choice%"=="2" (
    call :run_dev
    pause
    goto :menu
)
if "%choice%"=="3" (
    call :run_test
    pause
    goto :menu
)
if "%choice%"=="4" (
    call :run_build
    pause
    goto :menu
)
if "%choice%"=="5" (
    call :open_cases
    goto :menu
)
if "%choice%"=="6" (
    call :open_packs
    goto :menu
)
if "%choice%"=="7" goto :eof

echo Invalid option selected.
timeout /t 2 >nul
goto :menu

:run_exe
cls
if not exist "JustLegalOS.exe" if not exist "OpenSeason.exe" (
    if exist "src-tauri\target\release\open-season.exe" (
        copy /y "src-tauri\target\release\open-season.exe" "JustLegalOS.exe" >nul
        copy /y "src-tauri\target\release\open-season.exe" "OpenSeason.exe" >nul
    ) else (
        echo [WARN] JustLegalOS.exe not found. Building release binary first...
        call :run_build
    )
)
echo [INFO] Launching standalone JustLegalOS.exe...
if exist "JustLegalOS.exe" (
    start "" "JustLegalOS.exe"
) else (
    start "" "OpenSeason.exe"
)
exit /b 0

:run_dev
cls
echo [INFO] Starting JustLegal OS in dev mode (Vite + Tauri)...
echo [INFO] Press Ctrl+C in this terminal when finished to stop the server.
echo.
call npm run tauri dev
if errorlevel 1 (
    echo.
    echo [ERROR] Application exited with an error.
)
exit /b %ERRORLEVEL%

:run_test
cls
echo [INFO] Running agent end-to-end test suite...
echo.
if exist "C:\Program Files\Git\bin\bash.exe" (
    "C:\Program Files\Git\bin\bash.exe" examples\agent-e2e.sh
) else (
    bash examples\agent-e2e.sh
)
if errorlevel 1 (
    echo.
    echo [ERROR] Test run encountered issues.
) else (
    echo.
    echo [SUCCESS] End-to-end verification passed!
)
exit /b %ERRORLEVEL%

:run_build
cls
echo [INFO] Building production desktop executable...
echo.
call npm run tauri build -- --no-bundle
if errorlevel 1 (
    echo.
    echo [ERROR] Build failed.
) else (
    copy /y "src-tauri\target\release\open-season.exe" "JustLegalOS.exe" >nul
    copy /y "src-tauri\target\release\open-season.exe" "OpenSeason.exe" >nul
    if exist "src-tauri\target\release\justlegalos-cli.exe" (
        copy /y "src-tauri\target\release\justlegalos-cli.exe" "JustLegalOS-CLI.exe" >nul
        copy /y "src-tauri\target\release\justlegalos-cli.exe" "justlegalos-cli.exe" >nul
        copy /y "src-tauri\target\release\justlegalos-cli.exe" "openseason-cli.exe" >nul
    ) else (
        copy /y "src-tauri\target\release\openseason.exe" "JustLegalOS-CLI.exe" >nul
        copy /y "src-tauri\target\release\openseason.exe" "justlegalos-cli.exe" >nul
        copy /y "src-tauri\target\release\openseason.exe" "openseason-cli.exe" >nul
    )
    echo.
    echo [SUCCESS] Production build complete: JustLegalOS.exe and JustLegalOS-CLI.exe ready.
)
exit /b %ERRORLEVEL%

:open_cases
cls
set "CASES_DIR=%USERPROFILE%\Documents\JustLegal\Cases"
if not exist "%CASES_DIR%" mkdir "%CASES_DIR%"
echo [INFO] Opening Cases folder: %CASES_DIR%
start "" "%CASES_DIR%"
exit /b 0

:open_packs
cls
set "PACKS_DIR=%USERPROFILE%\Documents\JustLegal\Packs"
if not exist "%PACKS_DIR%" mkdir "%PACKS_DIR%"
echo [INFO] Opening Packs folder: %PACKS_DIR%
start "" "%PACKS_DIR%"
exit /b 0

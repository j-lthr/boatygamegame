@echo off
setlocal enabledelayedexpansion

:: Get the project name from Cargo.toml
for /f "tokens=2 delims== " %%a in ('findstr "^name" Cargo.toml') do (
    set PROJECT_NAME=%%a
    set PROJECT_NAME=!PROJECT_NAME:"=!
)

:: Set version (optional - extract from Cargo.toml)
for /f "tokens=2 delims== " %%a in ('findstr "^version" Cargo.toml') do (
    set VERSION=%%a
    set VERSION=!VERSION:"=!
)

echo Building release for !PROJECT_NAME! v!VERSION!

:: Build the release
echo Building Rust project...
cargo build --release
if errorlevel 1 (
    echo Build failed!
    pause
    exit /b 1
)

:: Create release directory
set RELEASE_DIR=release_!PROJECT_NAME!_!VERSION!
if exist !RELEASE_DIR! rmdir /s /q !RELEASE_DIR!
mkdir !RELEASE_DIR!

:: Copy executable(s)
echo Copying executables...
copy target\release\*.exe !RELEASE_DIR!\
if errorlevel 1 (
    echo No executables found in target\release\
    pause
    exit /b 1
)

:: Copy assets folder if it exists
if exist assets (
    echo Copying assets folder...
    xcopy assets !RELEASE_DIR!\assets\ /E /I /Q
) else (
    echo Warning: assets folder not found, skipping...
)

:: Copy additional files (optional)
if exist README.md copy README.md !RELEASE_DIR!\
if exist LICENSE copy LICENSE !RELEASE_DIR!\
if exist CHANGELOG.md copy CHANGELOG.md !RELEASE_DIR!\

:: Create zip file using PowerShell
set ZIP_NAME=!PROJECT_NAME!_!VERSION!_windows.zip
echo Creating zip file: !ZIP_NAME!

powershell -command "Compress-Archive -Path '!RELEASE_DIR!\*' -DestinationPath '!ZIP_NAME!' -Force"

if errorlevel 1 (
    echo Failed to create zip file!
    pause
    exit /b 1
)

:: Clean up temporary directory
rmdir /s /q !RELEASE_DIR!

echo.
echo ✓ Release created successfully: !ZIP_NAME!
echo.

pause
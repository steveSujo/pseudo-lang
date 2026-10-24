@echo off
REM Run all pseudo-lang scripts

cd /d "%~dp0.."

echo ========================================
echo Running pseudo-lang test scripts
echo ========================================

for %%f in (scripts\*.psudo) do (
    echo.
    echo ----------------------------------------
    echo Running: %%f
    echo ----------------------------------------
    cargo run -- -f "%%f"
)

echo.
echo ========================================
echo All scripts completed
echo ========================================
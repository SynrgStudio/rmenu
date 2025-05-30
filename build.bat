@echo off
echo Compilando rmenu...

REM Comprobar si Rust está instalado
where cargo >nul 2>&1
if %ERRORLEVEL% NEQ 0 (
    echo ERROR: Cargo/Rust no está instalado o no está en el PATH.
    echo Por favor, instala Rust desde https://www.rust-lang.org/tools/install
    exit /b 1
)

REM Compilar el proyecto
cargo build --release

if %ERRORLEVEL% NEQ 0 (
    echo Error al compilar rmenu.
    exit /b %ERRORLEVEL%
)

echo.
echo Compilación completada con éxito.
echo.
echo El ejecutable se encuentra en: target\release\rmenu.exe
echo El archivo de configuración de ejemplo se encuentra en: target\release\config_example.ini
echo.
echo Para ejecutar rmenu con opciones básicas:
echo target\release\rmenu.exe -elements "opcion1,opcion2,opcion3"
echo.
echo Para ver todas las opciones disponibles:
echo target\release\rmenu.exe -help
echo.

choice /C YN /M "¿Desea ejecutar rmenu ahora? (Y/N)"
if %ERRORLEVEL% EQU 1 (
    echo.
    echo Ejecutando rmenu con opciones de ejemplo...
    target\release\rmenu.exe -elements "opcion1,opcion2,opcion3,opcion4,opcion5" -prompt "Seleccione:"
)

exit /b 0 
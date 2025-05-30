@echo off
setlocal

REM Verificar si el script se ejecuta como administrador
net session >nul 2>&1
if %errorlevel% neq 0 (
    echo Este script requiere privilegios de administrador.
    echo Por favor, ejecuta este script como administrador.
    pause
    exit /b 1
)

REM Definir rutas de instalación
set INSTALL_DIR=%ProgramFiles%\rmenu
set CONFIG_DIR=%USERPROFILE%\.config\rmenu

REM Verificar si rmenu ya está compilado
if not exist "target\release\rmenu.exe" (
    echo No se encontró el ejecutable de rmenu.
    echo Compilando rmenu...
    call build.bat
    if %errorlevel% neq 0 (
        echo Error al compilar rmenu.
        pause
        exit /b 1
    )
)

REM Crear directorios de instalación
echo Creando directorios de instalación...
if not exist "%INSTALL_DIR%" mkdir "%INSTALL_DIR%"
if not exist "%CONFIG_DIR%" mkdir "%CONFIG_DIR%"

REM Copiar archivos
echo Copiando archivos...
copy /Y "target\release\rmenu.exe" "%INSTALL_DIR%\"
copy /Y "target\release\config_example.ini" "%INSTALL_DIR%\"
copy /Y "target\release\README.md" "%INSTALL_DIR%\"

REM Crear archivo de configuración en el directorio del usuario si no existe
if not exist "%CONFIG_DIR%\config.ini" (
    echo Creando archivo de configuración...
    copy /Y "target\release\config_example.ini" "%CONFIG_DIR%\config.ini"
)

REM Añadir al PATH si no está ya
echo Verificando PATH...
set "PATH_ENTRY=%INSTALL_DIR%"
call :check_path
if %errorlevel% neq 0 (
    echo Añadiendo rmenu al PATH del sistema...
    setx PATH "%PATH%;%PATH_ENTRY%" /M
    if %errorlevel% neq 0 (
        echo Error al añadir rmenu al PATH.
        echo Puedes añadirlo manualmente: %INSTALL_DIR%
    ) else (
        echo rmenu añadido al PATH correctamente.
    )
) else (
    echo rmenu ya está en el PATH.
)

echo.
echo Instalación completada.
echo.
echo rmenu se ha instalado en: %INSTALL_DIR%
echo Configuración guardada en: %CONFIG_DIR%
echo.
echo Puedes ejecutar rmenu desde cualquier terminal con el comando 'rmenu'.
echo.

pause
exit /b 0

:check_path
REM Función para verificar si la ruta ya está en el PATH
echo %PATH% | findstr /C:"%PATH_ENTRY%" >nul
if %errorlevel% equ 0 (
    exit /b 0
) else (
    exit /b 1
) 
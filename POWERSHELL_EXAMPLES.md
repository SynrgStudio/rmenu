# rmenu - Ejemplos de Scripts con PowerShell

## Introducción

`rmenu` está diseñado para ser una herramienta versátil que se puede integrar fácilmente en scripts de PowerShell. Al capturar la salida estándar (`stdout`) de `rmenu` y verificar su código de salida, los scripts pueden obtener selecciones del usuario de forma interactiva.

## Conceptos Clave para Scripts de PowerShell

1.  **Llamar a `rmenu.exe`:**
    Simplemente ejecuta `rmenu.exe` como cualquier otro comando. Asegúrate de que `rmenu.exe` esté en tu PATH o proporciona la ruta completa al ejecutable.

2.  **Capturar la Salida (`stdout`):
    La selección del usuario se envía a `stdout`. Puedes capturarla en una variable de PowerShell.
    ```powershell
    $seleccion = rmenu.exe -e "Opcion1,Opcion2"
    ```

3.  **Verificar el Código de Salida (`$LASTEXITCODE`):
    Inmediatamente después de que `rmenu.exe` termine, la variable automática `$LASTEXITCODE` contendrá:
    *   `0`: Si el usuario seleccionó un ítem presionando `Enter`.
    *   `1`: Si el usuario canceló la selección presionando `Escape` (o si la ventana se cerró por otros medios sin una selección explícita).
    ```powershell
    if ($LASTEXITCODE -eq 0) {
        # El usuario seleccionó algo
        Write-Host "Seleccionado: $seleccion"
    } else {
        # El usuario canceló
        Write-Host "Selección cancelada"
    }
    ```

4.  **Pasar Opciones a `rmenu`:**
    *   **Desde una lista de strings (para `-e`):
        ```powershell
        $misOpciones = "Rojo","Verde","Azul"
        $opcionesParaRmenu = $misOpciones -join ',' # Une con comas
        $seleccion = rmenu.exe -p "Color:" -e $opcionesParaRmenu
        ```
        O directamente si el string ya está formateado:
        ```powershell
        $opcionesString = "Alfa,Beta,Gamma"
        $seleccion = rmenu.exe -e $opcionesString
        ```

    *   **Desde `stdin` (un ítem por línea):
        ```powershell
        $misOpciones = "Uno`nDos`nTres"
        $seleccion = echo $misOpciones | rmenu.exe -p "Número:"
        ```
        O si cada opción es un elemento de un array:
        ```powershell
        $arrayOpciones = @("Laptop", "Desktop", "Tablet")
        $seleccion = $arrayOpciones | rmenu.exe -p "Dispositivo:"
        ```

## Ejemplos Prácticos

### 1. Selector de Navegador Web
Un script simple para elegir un navegador y lanzarlo.

```powershell
$opciones = "Firefox,Chrome,Edge,Brave"
$seleccion = rmenu.exe -p "Elige navegador:" -e $opciones

if ($LASTEXITCODE -eq 0 -and $seleccion) {
    Write-Host "Has seleccionado: $seleccion"
    try {
        switch ($seleccion) {
            "Firefox" { Start-Process firefox -ErrorAction Stop }
            "Chrome"  { Start-Process chrome -ErrorAction Stop }
            "Edge"    { Start-Process msedge -ErrorAction Stop }
            "Brave"   { Start-Process brave -ErrorAction Stop }
            default   { Write-Warning "Navegador no reconocido para lanzar." }
        }
    } catch {
        Write-Error "No se pudo lanzar '$seleccion'. Asegúrate de que está instalado y en el PATH."
    }
} else {
    Write-Host "Selección cancelada."
}
```

### 2. Lanzador de Aplicaciones (desde una lista fija)
Permite al usuario seleccionar una aplicación común para lanzarla.

```powershell
$apps = @{
    "Calculadora" = "calc.exe"
    "Bloc de notas" = "notepad.exe"
    "Explorador de archivos" = "explorer.exe"
    "Símbolo del sistema (CMD)" = "cmd.exe"
    "PowerShell" = "powershell.exe"
    "Paint" = "mspaint.exe"
}

# Convertir las claves del Hashtable (nombres de app) a un string delimitado por comas para rmenu
$nombresAppsParaRmenu = ($apps.Keys | ForEach-Object { $_ }) -join ','

$seleccionNombre = rmenu.exe -p "Lanzar aplicación:" -e $nombresAppsParaRmenu

if ($LASTEXITCODE -eq 0 -and $seleccionNombre) {
    $ejecutable = $apps[$seleccionNombre]
    if ($ejecutable) {
        Write-Host "Lanzando: $seleccionNombre ($ejecutable)"
        Start-Process $ejecutable
    } else {
        # Esto no debería ocurrir si $seleccionNombre viene de las claves de $apps
        Write-Error "Error interno: Aplicación '$seleccionNombre' no encontrada en la lista del script."
    }
} else {
    Write-Host "Lanzamiento cancelado."
}
```

### 3. Selector de Archivos en Directorio Actual (Nombres)
Muestra los nombres de los archivos en el directorio actual para selección.

```powershell
$archivos = Get-ChildItem -File -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Name

if ($null -eq $archivos -or $archivos.Count -eq 0) {
    Write-Host "No se encontraron archivos en el directorio actual."
    exit
}

# Los nombres de archivo se pasan uno por línea a stdin
$seleccion = $archivos | rmenu.exe -p "Selecciona un archivo:"

if ($LASTEXITCODE -eq 0 -and $seleccion) {
    Write-Host "Archivo seleccionado: $seleccion"
    # Acción de ejemplo: Abrir con el programa predeterminado
    # Invoke-Item -Path (Join-Path -Path (Get-Location) -ChildPath $seleccion)
    
    # Acción de ejemplo: Abrir con Notepad
    # notepad.exe (Join-Path -Path (Get-Location) -ChildPath $seleccion)
} else {
    Write-Host "Selección cancelada."
}
```

### 4. Selector de Tema (Ejemplo Conceptual)
Un script que podría cambiar el tema de una aplicación (simulado).

```powershell
$temasDisponibles = "Claro,Oscuro,Azul Metálico,Verde Bosque"
$temaActual = "Claro" # Simular lectura de configuración actual

$seleccionTema = rmenu.exe -p "Tema actual: $temaActual. Elige nuevo tema:" -e $temasDisponibles

if ($LASTEXITCODE -eq 0 -and $seleccionTema) {
    if ($seleccionTema -ne $temaActual) {
        Write-Host "Cambiando tema a: $seleccionTema"
        # Aquí iría la lógica para aplicar el cambio de tema
        # Ejemplo: Set-AppTheme -Name $seleccionTema
        $temaActual = $seleccionTema
        Write-Host "Tema cambiado a $temaActual."
    } else {
        Write-Host "Has seleccionado el tema actual. No se realizaron cambios."
    }
} else {
    Write-Host "Cambio de tema cancelado."
}
```

## Consideraciones Adicionales

*   **Modo Silencioso (`-s`):**
    Si tu script necesita una operación completamente limpia donde los mensajes de error de `rmenu` (como problemas al cargar `config.ini`) no deben aparecer en `stderr`, puedes usar `rmenu.exe -s ...`.

*   **Manejo de Strings y Caracteres Especiales:**
    *   Al pasar elementos a `-e`, si tus elementos contienen el delimitador (coma por defecto), `rmenu` los interpretará como elementos separados. Si un elemento *debe* contener una coma literal, necesitarás cambiar el `element_delimiter` en `config.ini` a otro carácter y usar ese nuevo delimitador.
    *   PowerShell maneja el entrecomillado y los espacios en los argumentos pasados a ejecutables externos. Generalmente, si una variable contiene un string con espacios, PowerShell lo pasará correctamente.

*   **Rendimiento con Listas Muy Grandes:**
    `rmenu` está diseñado para ser rápido. Sin embargo, pasar cientos de miles de opciones podría tener implicaciones de rendimiento, especialmente en la forma en que se pasan los datos (a través de `-e` vs. `stdin`). Para listas extremadamente largas, considera si `rmenu` es la herramienta más adecuada o si se necesita un pre-filtrado.

*   **Codificación de Caracteres:**
    PowerShell y `rmenu` (que internamente usa Rust Strings / UTF-8) generalmente manejan bien los caracteres Unicode. Asegúrate de que tu terminal y tus scripts estén configurados para manejar la codificación que necesitas si trabajas con caracteres no ASCII.

Estos ejemplos deberían servir como un buen punto de partida para integrar `rmenu` en tus propios scripts de PowerShell. 
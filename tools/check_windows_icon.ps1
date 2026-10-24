param(
    [Parameter(Mandatory = $true)][string]$Executable,
    [Parameter(Mandatory = $true)][string]$Destination,
    [Parameter(Mandatory = $true)][string]$Version
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
$metadata = [System.Diagnostics.FileVersionInfo]::GetVersionInfo($Executable)
if ($metadata.FileVersion -ne $Version -or $metadata.ProductVersion -ne $Version) {
    throw 'Native Windows executable version resource differs from package version'
}
if ($metadata.ProductName -ne 'Oh My Lemmings') {
    throw 'Native Windows product metadata is missing'
}
$appIcon = [System.Drawing.Icon]::ExtractAssociatedIcon($Executable)
if ($null -eq $appIcon) { throw 'Windows could not extract the executable icon' }
try {
    $bitmap = $appIcon.ToBitmap()
    try {
        $bitmap.Save($Destination, [System.Drawing.Imaging.ImageFormat]::Png)
        @{
            status = 'PASS'
            version = $metadata.FileVersion
            product_version = $metadata.ProductVersion
            width = $bitmap.Width
            height = $bitmap.Height
            native_shell_icon = $true
        } | ConvertTo-Json -Compress
    } finally { $bitmap.Dispose() }
} finally { $appIcon.Dispose() }

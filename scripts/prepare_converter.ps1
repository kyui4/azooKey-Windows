param(
    [string]$Checkout = (Join-Path $PSScriptRoot '../server-swift/.build/checkouts/AzooKeyKanaKanjiConverter')
)
$ErrorActionPreference = 'Stop'
$revision = '7d5dd99fd7f4d1251ff94ae6c75642e4b36f327b'
$head = & git -C $Checkout rev-parse HEAD
if ($LASTEXITCODE -ne 0 -or $head -ne $revision) {
    throw "GPU patch requires converter revision $revision (found $head)."
}
$path = Join-Path $Checkout 'Sources/KanaKanjiConverterModule/Zenz/ZenzContext.swift'
$source = [IO.File]::ReadAllText($path)
$marker = '// azooKey-Windows GPU offload v1'
if ($source.Contains($marker)) { Write-Output 'GPU patch already applied'; return }
$anchor = '        model_params.use_mmap = true'
if (($source.Split([string[]]@($anchor), [StringSplitOptions]::None)).Length -ne 2) {
    throw 'Converter source changed: GPU patch anchor is not unique.'
}
$patch = @'
        model_params.use_mmap = true
        // azooKey-Windows GPU offload v1
        #if canImport(llama)
        let requestedLayers = max(0, min(99, Int32(ProcessInfo.processInfo.environment["AZOOKEY_GPU_LAYERS"] ?? "0") ?? 0))
        let gpuAvailable = llama_supports_gpu_offload()
        model_params.n_gpu_layers = gpuAvailable ? requestedLayers : 0
        print("Zenzai: requested GPU layers = \(requestedLayers), GPU backend available = \(gpuAvailable)")
        if requestedLayers > 0 && !gpuAvailable {
            print("Zenzai: GPU backend unavailable; falling back to CPU")
        }
        #endif
'@
$source = $source.Replace($anchor, $patch.Replace("`r`n", "`n"))
[IO.File]::WriteAllText($path, $source, [Text.UTF8Encoding]::new($false))
Write-Output 'Applied GPU offload patch to the pinned converter'

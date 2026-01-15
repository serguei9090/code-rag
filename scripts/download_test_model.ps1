$Repo = "Xenova/bge-small-en-v1.5"
$BaseUrl = "https://huggingface.co/$Repo/resolve/main"
$TargetDir = "tests/fixtures/models/bge-small-en-v1.5"

if (-not (Test-Path $TargetDir)) {
    New-Item -ItemType Directory -Force -Path $TargetDir
}

$Files = @(
    @{ Src = "onnx/model.onnx"; Dest = "model.onnx" },
    @{ Src = "tokenizer.json"; Dest = "tokenizer.json" },
    @{ Src = "config.json"; Dest = "config.json" },
    @{ Src = "tokenizer_config.json"; Dest = "tokenizer_config.json" },
    @{ Src = "special_tokens_map.json"; Dest = "special_tokens_map.json" }
)

foreach ($File in $Files) {
    $Url = "$BaseUrl/$($File.Src)"
    $OutFile = Join-Path $TargetDir $File.Dest
    if (-not (Test-Path $OutFile)) {
        Write-Host "Downloading $Url to $OutFile..."
        Invoke-WebRequest -Uri $Url -OutFile $OutFile
    } else {
        Write-Host "$OutFile already exists, skipping."
    }
}

Write-Host "Model download complete."

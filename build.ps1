$ErrorActionPreference = "Stop"
& python "$PSScriptRoot/scripts/build.py" @args
exit $LASTEXITCODE

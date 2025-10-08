# Process GitHub Forks into Workspace
param([string]$OrgName='FlexNetOS',[string[]]$ForkNames=@(),[switch]$ListOnly,[switch]$DryRun)
$WorkspaceRoot='D:\dev\workspaces\noa_ark_os'
$ForksDir=Join-Path $WorkspaceRoot 'crc\drop-in\incoming\forks'

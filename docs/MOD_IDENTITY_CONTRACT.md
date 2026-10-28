# TFL Client identity contract

Launcher and Minecraft mod share the public name TFL Client and local Nunito typography.
The mod retains loader ID tflcore and config/tflcore/ for compatibility. They remain separate
processes and repositories; this change adds no IPC, credentials exchange or shared config writer.
The existing modpack catalog API, file paths and internal TflSelection component names stay stable.
Release-note history retains original names. No TRS Client/Launcher branding was found in the
launcher source audited at ae24e91. Screenshots/clips in the Minecraft mod belong to TFLClientMOD.

Default typography is Nunito only when no saved choice exists; explicit system/other font
preferences remain available. This does not migrate or overwrite any existing localStorage value.

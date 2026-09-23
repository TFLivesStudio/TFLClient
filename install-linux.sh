#!/usr/bin/env bash
# Instalador de TFL Client para Linux — detecta el sistema y usa el
# paquete correcto (deb/rpm/AppImage) de la última release en GitHub.
# Uso:
#   curl -fsSL https://raw.githubusercontent.com/TFLivesStudio/TFLClient/main/install-linux.sh | bash
set -euo pipefail

REPO="TFLivesStudio/TFLClient"
BASE_URL="https://github.com/${REPO}/releases/latest/download"
TMP_DIR="$(mktemp -d)"
trap 'rm -rf "$TMP_DIR"' EXIT

info() { printf '\033[1;34m==>\033[0m %s\n' "$1"; }
ok()   { printf '\033[1;32m✓\033[0m %s\n' "$1"; }
err()  { printf '\033[1;31mError:\033[0m %s\n' "$1" >&2; }

install_deb() {
	info "Detectado sistema basado en Debian/Ubuntu (apt) — bajando paquete .deb…"
	curl -fsSL -o "$TMP_DIR/TFLClient.deb" "$BASE_URL/TFLClient-Linux.deb"
	info "Instalando (te va a pedir tu contraseña de administrador)…"
	sudo apt install -y "$TMP_DIR/TFLClient.deb"
	ok "Listo — buscá \"TFL Client\" en el menú de aplicaciones."
}

install_rpm() {
	info "Detectado sistema basado en Fedora/RHEL — bajando paquete .rpm…"
	curl -fsSL -o "$TMP_DIR/TFLClient.rpm" "$BASE_URL/TFLClient-Linux.rpm"
	info "Instalando (te va a pedir tu contraseña de administrador)…"
	if command -v dnf >/dev/null 2>&1; then
		sudo dnf install -y "$TMP_DIR/TFLClient.rpm"
	else
		sudo rpm -i "$TMP_DIR/TFLClient.rpm"
	fi
	ok "Listo — buscá \"TFL Client\" en el menú de aplicaciones."
}

install_appimage() {
	info "No se detectó apt ni dnf — instalando como AppImage (funciona en cualquier distro)…"
	local install_dir="$HOME/.local/share/tflclient"
	local bin_dir="$HOME/.local/bin"
	local apps_dir="$HOME/.local/share/applications"
	local icon_dir="$HOME/.local/share/icons"
	mkdir -p "$install_dir" "$bin_dir" "$apps_dir" "$icon_dir"

	curl -fsSL -o "$install_dir/TFLClient.AppImage" "$BASE_URL/TFLClient-Linux.AppImage"
	chmod +x "$install_dir/TFLClient.AppImage"

	curl -fsSL -o "$icon_dir/tflclient.png" \
		"https://raw.githubusercontent.com/${REPO}/main/src-tauri/icons/128x128.png" \
		2>/dev/null || true

	# Symlink al PATH si ~/.local/bin ya está agregado (Ubuntu lo agrega
	# solo si la carpeta existe al iniciar sesión) — si no, igual queda
	# accesible desde el menú de aplicaciones por el .desktop de abajo.
	ln -sf "$install_dir/TFLClient.AppImage" "$bin_dir/tflclient"

	cat > "$apps_dir/tflclient.desktop" <<-EOF
	[Desktop Entry]
	Type=Application
	Name=TFL Client
	Comment=Launcher de Minecraft
	Exec=$install_dir/TFLClient.AppImage
	Icon=$icon_dir/tflclient.png
	Categories=Game;
	Terminal=false
	EOF
	chmod +x "$apps_dir/tflclient.desktop"

	ok "Listo — buscá \"TFL Client\" en el menú de aplicaciones, o corré \"tflclient\" en una terminal nueva."
}

main() {
	if command -v apt >/dev/null 2>&1; then
		install_deb
	elif command -v dnf >/dev/null 2>&1 || command -v rpm >/dev/null 2>&1; then
		install_rpm
	else
		install_appimage
	fi
}

main

// Browser-only IPC fixture. Never imported by production code.
export function installFixture({
	quality = 'Balanced',
	appVersion = '0.12.4-beta.1',
	empty = false,
	language = 'es',
	theme = 'dark',
	premium = false,
	skin = false
} = {}) {
	const user = {
		username: 'Alex',
		uuid: 'fixture-account',
		user_type: premium ? 'Premium' : 'Cracked'
	};
	const instance = (name, id, loader = 'fabric', server = null) => ({
		uuid: id,
		name,
		mc_version: '1.21.1',
		loader,
		loader_version: '0.16.10',
		launch_version_id: '1.21.1',
		last_played: 1729000000,
		play_time_secs: 12600,
		min_memory: null,
		max_memory: null,
		server_type: server,
		server_jar: server ? 'server.jar' : null,
		server_build: null
	});
	const instances = empty
		? []
		: [
				instance('Horizontes', 'horizons'),
				instance('Create · New beginnings', 'create', 'forge'),
				instance('Vanilla stories', 'vanilla', 'vanilla'),
				instance('TFL Friends', 'server', 'vanilla', 'paper')
			];
	let settings = {
		user: [user],
		active_user_idx: 0,
		min_memory: 2048,
		max_memory: 4096,
		jre8_path: '',
		jre17_path: '',
		jre21_path: '',
		jre25_path: '',
		language,
		theme,
		quality_profile: quality,
		discord_presence: false,
		onboarded: true,
		jvm_args: '',
		env_vars: {},
		reduce_animations: false,
		disable_blur_effects: false,
		disable_infinite_animations: false,
		auto_updates: false,
		beta_updates: false,
		native_dialog_mode: 'auto',
		native_dialog_mode_prompted: true,
		server_connection_info_shown: true,
		multi_instance_warning_dismissed: false,
		sidebar_collapsed: false
	};
	const texture = document.createElement('canvas');
	texture.width = texture.height = 64;
	const ctx = texture.getContext('2d');
	ctx.fillStyle = '#e0b493';
	ctx.fillRect(0, 0, 64, 64);
	ctx.fillStyle = '#366cad';
	ctx.fillRect(16, 16, 24, 16);
	const skinUrl = texture.toDataURL();
	const callbacks = new Map(),
		listeners = new Map();
	let sequence = 0;
	window.__fixtureCommands = [];
	window.__fixtureEmit = (type, data) => {
		for (const cb of listeners.values())
			callbacks.get(cb)?.({ event: 'app-event', id: 1, payload: { type, data } });
	};
	window.__TAURI_EVENT_PLUGIN_INTERNALS__ = { unregisterListener: () => {} };
	window.__TAURI_INTERNALS__ = {
		metadata: { currentWindow: { label: 'main' }, currentWebview: { label: 'main' } },
		convertFileSrc: (p) => p,
		transformCallback: (fn) => {
			const id = ++sequence;
			callbacks.set(id, fn);
			return id;
		},
		unregisterCallback: (id) => callbacks.delete(id),
		invoke: async (cmd, args = {}) => {
			window.__fixtureCommands.push({ cmd, args });
			if (cmd === 'plugin:event|listen') {
				listeners.set(++sequence, args.handler);
				return sequence;
			}
			if (cmd === 'plugin:event|unlisten') {
				listeners.delete(args.eventId);
				return null;
			}
			if (cmd === 'get_settings') return settings;
			if (cmd === 'update_settings') {
				settings = args.newSettings;
				return null;
			}
			if (cmd === 'set_quality_profile') {
				settings = { ...settings, quality_profile: args.profile };
				return settings;
			}
			if (cmd === 'switch_user') return user;
			if (cmd === 'get_current_user') return user;
			if (cmd === 'get_user_list') return [user];
			if (cmd === 'check_mod_updates' || cmd === 'find_duplicate_mods') return [];
			if (cmd === 'get_instances') return instances;
			if (cmd === 'get_system_status')
				return {
					java_major: 21,
					ram_assigned_mb: 4096,
					ram_total_mb: 16384,
					disk_free_gb: 120,
					disk_total_gb: 500,
					instances_dir: '/fixture/instances'
				};
			if (cmd.startsWith('get_recommended_ram'))
				return { total_mb: 16384, recommended_min_mb: 2048, recommended_max_mb: 4096 };
			if (cmd === 'get_recent_activity')
				return [
					{
						id: 'session-1',
						instance_name: 'Horizontes',
						mc_version: '1.21.1',
						loader: 'fabric',
						duration_secs: 3600,
						ended_at: 1729000000
					}
				];
			if (cmd === 'get_favorite_servers')
				return [{ id: 'fav-1', name: 'Nuestra comunidad', address: 'play.example.test' }];
			if (cmd === 'get_mojang_profile')
				return {
					skins: skin
						? [{ id: 'fixture-skin', state: 'ACTIVE', url: skinUrl, variant: 'CLASSIC' }]
						: [],
					capes: []
				};
			if (cmd === 'get_java_status')
				return [8, 17, 21, 25].map((major) => ({ major, installed: major === 21, path: null }));
			if (cmd === 'get_server_connection_info')
				return {
					local_ip: '192.0.2.1',
					public_ip: null,
					port: 25565,
					port_forwarded: false,
					tunnel_address: null
				};
			if (cmd === 'get_playit_status') return false;
			if (cmd === 'get_minecraft_versions')
				return [{ id: '1.21.1', type: 'release', release_time: '2024-08-08' }];
			if (cmd.includes('is_maximized') || cmd.includes('is_running')) return false;
			if (cmd.includes('icon_path') || cmd.includes('wallpaper') || cmd.includes('skin_texture'))
				return null;
			if (cmd.startsWith('get_') || cmd.startsWith('list_') || cmd.startsWith('search_')) return [];
			return null;
		}
	};
	localStorage.setItem('tfl-last-seen-version', appVersion);
	localStorage.setItem('tfl-locale', language);
}

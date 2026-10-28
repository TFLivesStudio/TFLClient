// Browser-only IPC fixture. Never imported by production code.
export function installFixture({
	quality = 'Balanced',
	appVersion = '0.14.1',
	empty = false,
	language = 'es',
	theme = 'dark',
	premium = false,
	skin = false,
	username = 'Alex',
	collapsed = false,
	logWindow = false,
	telemetry = false,
	iconPresets = []
} = {}) {
	if (logWindow) window.__TFL_LOG_INSTANCE__ = 'Horizontes';
	let samples = 0;
	const installedContent = new Map(),
		localMedia = [];
	window.__fixtureRunning = [];
	const user = {
		username,
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
		sidebar_collapsed: collapsed
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
			if (cmd === 'get_running_instance_stats' || cmd === 'get_running_server_stats') {
				if (!telemetry || window.__fixtureStatsMode === 'missing') return null;
				samples++;
				return {
					cpu_core_percent: 250 + samples * 5,
					cpu_percent: (250 + samples * 5) / 8,
					logical_cores: 8,
					memory_mb: 2048 + samples * 16,
					disk_bytes_delta: 1048576,
					sampled_at_ms: Date.now(),
					sample_interval_ms: 1000,
					ready: samples > 1
				};
			}
			if (
				[
					'search_mods',
					'search_shaders',
					'search_resourcepacks',
					'search_plugins',
					'search_modpacks'
				].includes(cmd)
			) {
				const offset = args.offset ?? 0,
					limit = args.limit ?? 20;
				return {
					hits: Array.from({ length: Math.min(limit, 45 - offset) }, (_, i) => ({
						project_id: `project-${offset + i}`,
						title: `Fixture project ${offset + i + 1}`,
						description: 'Compatible content from the paginated catalogue.',
						icon_url: null,
						downloads: 10000,
						author: 'Fixture'
					})),
					total_hits: 45,
					offset,
					limit
				};
			}
			if (/^get_(mod|shader|resourcepack|plugin)_versions$/.test(cmd))
				return [{ id: 'explicit-version', version_number: '1.2.3', name: 'Compatible version' }];
			if (cmd.endsWith('_project_detail'))
				return {
					project_id: args.projectId,
					title: 'Fixture detail',
					description: 'Project description',
					body: '## Full project details\n\nSafe **Markdown** content.',
					categories: ['optimization'],
					game_versions: ['1.21.1'],
					loaders: ['fabric'],
					downloads: 10000,
					followers: 100,
					published: '2026-01-01',
					updated: '2026-10-01',
					source_url: null,
					wiki_url: null,
					discord_url: null,
					issues_url: null,
					gallery: []
				};
			if (cmd === 'get_running_instances') return window.__fixtureRunning;
			if (cmd === 'list_instance_icon_presets') return iconPresets;
			if (cmd === 'list_scene_media') return localMedia;
			if (cmd === 'pick_scene_video_file') return '/fixture/aurora.webm';
			if (cmd === 'import_scene_video') {
				if (window.__fixtureMediaError) throw new Error('Unsupported or oversized video');
				const item = {
					id: 'scene-123456abcdef',
					path: '/scenes/aurora.webm',
					bytes: 126000,
					mime: 'video/webm'
				};
				localMedia.push(item);
				return item;
			}
			if (cmd === 'remove_scene_media') {
				const index = localMedia.findIndex((m) => m.id === args.id);
				if (index >= 0) localMedia.splice(index, 1);
				return;
			}
			if (
				['install_mod', 'install_shader', 'install_resourcepack', 'install_plugin'].includes(cmd)
			) {
				await new Promise((r) => setTimeout(r, 180));
				if (window.__fixtureInstallFailure) throw new Error('Fixture install failed');
				installedContent.set(args.projectId, {
					project_id: args.projectId,
					filename: args.projectId + '.jar',
					display_name: args.projectId,
					enabled: true,
					version_id: 'v1',
					version_number: '1',
					icon_url: null
				});
				return;
			}
			if (/^get_installed_(mods|shaders|resourcepacks|plugins)_info$/.test(cmd))
				return [...installedContent.values()];
			if (cmd === 'get_skin_texture_url') return skin ? skinUrl : null;
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

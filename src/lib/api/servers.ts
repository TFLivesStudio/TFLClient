// Instancias de servidor: consola, mundos, archivos, conexión, playit.gg, favoritos y ping
import { invoke } from './invoke';
import type {
	InstanceData,
	ServerType,
	WorldInfo,
	FileEntry,
	ServerConnectionInfo,
	PlayitClaimInfo,
	FavoriteServer,
	ServerStatus
} from '$lib/types/types';

export const getServerVersions = (serverType: ServerType) =>
	invoke<string[]>('get_server_versions', { serverType });
export const getServerConnectionInfo = (instanceName: string) =>
	invoke<ServerConnectionInfo>('get_server_connection_info', { instanceName });
export const createServerInstance = (name: string, mcVersion: string, serverType: ServerType) =>
	invoke<InstanceData>('create_server_instance', { name, mcVersion, serverType });
export const launchServer = (instanceName: string) =>
	invoke<void>('launch_server', { instanceName });
export const stopServer = (instanceName: string) => invoke<void>('stop_server', { instanceName });
export const sendServerCommand = (instanceName: string, command: string) =>
	invoke<void>('send_server_command', { instanceName, command });
export const isServerRunning = (instanceName: string) =>
	invoke<boolean>('is_server_running', { instanceName });
export const listServerWorlds = (instanceName: string) =>
	invoke<WorldInfo[]>('list_server_worlds', { instanceName });
export const deleteServerWorld = (instanceName: string, worldName: string) =>
	invoke<void>('delete_server_world', { instanceName, worldName });
export const listInstanceDir = (instanceName: string, subpath: string) =>
	invoke<FileEntry[]>('list_instance_dir', { instanceName, subpath });
export const readInstanceTextFile = (instanceName: string, subpath: string) =>
	invoke<string>('read_instance_text_file', { instanceName, subpath });
export const writeInstanceTextFile = (instanceName: string, subpath: string, content: string) =>
	invoke<void>('write_instance_text_file', { instanceName, subpath, content });
export const deleteInstancePath = (instanceName: string, subpath: string) =>
	invoke<void>('delete_instance_path', { instanceName, subpath });
export const createInstanceDir = (instanceName: string, subpath: string) =>
	invoke<void>('create_instance_dir', { instanceName, subpath });
export const playitIsLinked = () => invoke<boolean>('playit_is_linked');
export const playitStartClaim = () => invoke<PlayitClaimInfo>('playit_start_claim');
export const playitPollClaim = (code: string) => invoke<string>('playit_poll_claim', { code });
export const playitUnlink = () => invoke<void>('playit_unlink');
export const playitOpenClaimUrl = (code: string) => invoke<void>('playit_open_claim_url', { code });
export const getFavoriteServers = () => invoke<FavoriteServer[]>('get_favorite_servers');
export const addFavoriteServer = (name: string, address: string) =>
	invoke<FavoriteServer>('add_favorite_server', { name, address });
export const removeFavoriteServer = (id: string) => invoke<void>('remove_favorite_server', { id });
export const pingServer = (address: string) => invoke<ServerStatus>('ping_server', { address });

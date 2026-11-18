import { invoke } from './invoke';

export type McpClient = 'codex' | 'opencode' | 'claude';
export type McpTarget = {
	id: string;
	client: McpClient;
	label: string;
	path: string;
	installed: boolean;
	supported: boolean;
	configured: boolean;
	managed: boolean;
	revision: string;
	reason: string | null;
};

export const getMcpTargets = () => invoke<McpTarget[]>('mcp_quick_connect_targets');
export const writeMcpTarget = (target: McpTarget, enabled: boolean) =>
	invoke<McpTarget[]>('mcp_quick_connect_write', {
		targetId: target.id,
		enabled,
		expectedRevision: target.revision
	});

export const MCP_CHECK_PROMPT = {
	es: 'Comprueba si tienes disponibles las herramientas MCP de TFL Client y verifica la conexión con una consulta de solo lectura. Salúdame, indica si pudiste comprobar la conexión, explica qué acciones permiten las herramientas y permisos actuales y dame tres ejemplos útiles. Si no puedes comprobarla, dilo claramente y explica cómo conectar. No modifiques nada ni muestres tokens o datos privados.',
	en: 'Check whether TFL Client MCP tools are available and verify the connection with a read-only query. Greet me, say whether you could verify the connection, explain which actions the current tools and permissions allow, and give me three useful examples. If you cannot verify it, say so clearly and explain how to connect. Do not modify anything or show tokens or private data.'
};

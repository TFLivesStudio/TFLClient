// Estado compartido de exploración de contenido (mods, shaders, resource
// packs, plugins, modpacks) — antes vivía entero e inline dentro de
// ContentManager.svelte (pestaña Descargar). Se saca a una clase para que
// la vista compacta (ContentManager.svelte) Y la vista amplia
// (ContentExplorerView.svelte) consuman la MISMA lógica de búsqueda sin
// duplicarla — cada vista crea su propia instancia al montarse (no hay
// estado compartido EN VIVO entre las dos, para que paginar en una no le
// mueva la página a la otra sin que el usuario lo vea).
//
// Patrón: una clase con campos `$state` — el patrón que recomienda Svelte 5
// para encapsular estado reactivo fuera de un componente (a diferencia de
// un objeto plano devuelto por una función, donde desestructurar los
// campos perdería la reactividad).
import { untrack } from 'svelte';
import type {
	InstanceData,
	ModSearchHit,
	ModVersionSummary,
	ContentProjectDetail,
	SortIndex
} from '$lib/types/types';
import {
	searchMods,
	installMod,
	getModVersions,
	getModProjectDetail,
	searchShaders,
	installShader,
	getShaderVersions,
	getShaderProjectDetail,
	searchResourcepacks,
	installResourcepack,
	getResourcepackVersions,
	getResourcepackProjectDetail,
	searchPlugins,
	installPlugin,
	getPluginVersions,
	getPluginProjectDetail,
	DEFAULT_PAGE_SIZE
} from '$lib/api/content';
import { searchModpacks, installModpack, getModpackProjectDetail } from '$lib/api/modpacks';

export type ExplorerKind = 'mod' | 'shader' | 'resourcepack' | 'plugin' | 'modpack';

/** Causa de un error de búsqueda, para mostrar el mensaje/acción correctos
 * — ver `ModrinthProvider::search` en Rust, que antepone uno de estos
 * prefijos al error según el tipo de falla HTTP antes de propagarlo. */
export type SearchErrorKind = 'network' | 'timeout' | 'invalid' | 'rate-limited' | 'provider-down';

const ERROR_PREFIXES: Record<string, SearchErrorKind> = {
	'RATE_LIMITED:': 'rate-limited',
	'PROVIDER_DOWN:': 'provider-down',
	'TIMEOUT:': 'timeout',
	'INVALID_RESPONSE:': 'invalid'
};

function classifyError(e: unknown): { kind: SearchErrorKind; message: string } {
	const raw = String(e instanceof Error ? e.message : e);
	for (const [prefix, kind] of Object.entries(ERROR_PREFIXES)) {
		if (raw.startsWith(prefix)) {
			return { kind, message: raw.slice(prefix.length).trim() || raw };
		}
	}
	return { kind: 'network', message: raw };
}

/** La clave de `t()` a mostrar para cada causa de error — centralizado acá
 * porque tanto ContentManager.svelte como ContentExplorerView.svelte
 * necesitan el mismo mapeo. */
export function errorMessageKey(
	kind: SearchErrorKind
):
	| 'contentManager.errorNetwork'
	| 'contentManager.errorTimeout'
	| 'contentManager.errorInvalid'
	| 'contentManager.errorRateLimited'
	| 'contentManager.errorProviderDown' {
	switch (kind) {
		case 'network':
			return 'contentManager.errorNetwork';
		case 'timeout':
			return 'contentManager.errorTimeout';
		case 'invalid':
			return 'contentManager.errorInvalid';
		case 'rate-limited':
			return 'contentManager.errorRateLimited';
		case 'provider-down':
			return 'contentManager.errorProviderDown';
	}
}

/** Contexto que el composable necesita en cada operación — se pide vía
 * función (no un valor fijo tomado una sola vez) porque `instance` puede
 * cambiar mientras el componente sigue montado (el launcher no remonta
 * ContentManager/ContentExplorerView al cambiar de instancia activa, solo
 * actualiza el prop — ver el `$effect` equivalente que ya existía antes de
 * esta refactorización). */
export interface ContentExplorerContext {
	instance: InstanceData;
	kind: ExplorerKind;
}
type GetContext = () => ContentExplorerContext;

function serverTypeFor(instance: InstanceData) {
	return instance.server_type ?? 'paper';
}

function dispatchSearch(
	ctx: ContentExplorerContext,
	query: string,
	categories: string[],
	offset: number,
	index: SortIndex
) {
	const { instance, kind } = ctx;
	switch (kind) {
		case 'mod':
			return searchMods(
				query,
				instance.mc_version,
				instance.loader,
				categories,
				offset,
				DEFAULT_PAGE_SIZE,
				index
			);
		case 'shader':
			return searchShaders(
				query,
				instance.mc_version,
				categories,
				offset,
				DEFAULT_PAGE_SIZE,
				index
			);
		case 'resourcepack':
			return searchResourcepacks(
				query,
				instance.mc_version,
				categories,
				offset,
				DEFAULT_PAGE_SIZE,
				index
			);
		case 'plugin':
			return searchPlugins(
				query,
				instance.mc_version,
				serverTypeFor(instance),
				categories,
				offset,
				DEFAULT_PAGE_SIZE,
				index
			);
		case 'modpack':
			return searchModpacks(
				query,
				instance.mc_version,
				instance.loader,
				categories,
				offset,
				DEFAULT_PAGE_SIZE,
				index
			);
	}
}

function dispatchInstall(
	ctx: ContentExplorerContext,
	projectId: string,
	versionId: string | undefined
) {
	const { instance, kind } = ctx;
	switch (kind) {
		case 'mod':
			return installMod(instance.name, projectId, instance.mc_version, instance.loader, versionId);
		case 'shader':
			return installShader(instance.name, projectId, instance.mc_version, versionId);
		case 'resourcepack':
			return installResourcepack(instance.name, projectId, instance.mc_version, versionId);
		case 'plugin':
			return installPlugin(
				instance.name,
				projectId,
				instance.mc_version,
				serverTypeFor(instance),
				versionId
			);
		case 'modpack':
			// Los modpacks no tienen dropdown de versiones (ver
			// ModpacksPanel.svelte original) — siempre la última compatible,
			// `versionId` nunca llega poblado para este kind.
			return installModpack(instance.name, projectId, instance.mc_version, instance.loader);
	}
}

function dispatchVersions(
	ctx: ContentExplorerContext,
	projectId: string
): Promise<ModVersionSummary[]> {
	const { instance, kind } = ctx;
	switch (kind) {
		case 'mod':
			return getModVersions(projectId, instance.mc_version, instance.loader);
		case 'shader':
			return getShaderVersions(projectId, instance.mc_version);
		case 'resourcepack':
			return getResourcepackVersions(projectId, instance.mc_version);
		case 'plugin':
			return getPluginVersions(projectId, instance.mc_version, serverTypeFor(instance));
		case 'modpack':
			// Sin dropdown de versiones para modpacks — ver dispatchInstall.
			return Promise.resolve([]);
	}
}

function dispatchProjectDetail(
	ctx: ContentExplorerContext,
	projectId: string
): Promise<ContentProjectDetail> {
	switch (ctx.kind) {
		case 'mod':
			return getModProjectDetail(projectId);
		case 'shader':
			return getShaderProjectDetail(projectId);
		case 'resourcepack':
			return getResourcepackProjectDetail(projectId);
		case 'plugin':
			return getPluginProjectDetail(projectId);
		case 'modpack':
			return getModpackProjectDetail(projectId);
	}
}

const DETAIL_CACHE_TTL_MS = 10 * 60 * 1000;

export class ContentExplorer {
	private getCtx: GetContext;

	// ── Búsqueda / paginación / orden ──────────────────────────────────
	query = $state('');
	selectedCategories = $state<Set<string>>(new Set());
	/** Sin búsqueda de texto, Relevancia no significa nada — se arranca en
	 * "más descargados" (pedido del cliente) y se corrige sola si el
	 * usuario tipea algo (ver `setQuery`). */
	sortIndex = $state<SortIndex>('downloads');
	offset = $state(0);
	readonly limit = DEFAULT_PAGE_SIZE;
	results = $state<ModSearchHit[]>([]);
	totalHits = $state(0);
	loading = $state(false);
	errorKind = $state<SearchErrorKind | null>(null);
	errorMessage = $state<string | null>(null);

	// ── Instalación / versiones ────────────────────────────────────────
	installingId = $state<string | null>(null);
	installError = $state<string | null>(null);
	openVersionsFor = $state<string | null>(null);
	versionOptions = $state<ModVersionSummary[]>([]);
	loadingVersions = $state(false);

	// ── Ficha de proyecto (sub-vista de detalle) ───────────────────────
	detailProjectId = $state<string | null>(null);
	/** Snapshot del `ModSearchHit` desde el que se abrió — ver `openDetail`. */
	detailHit = $state<ModSearchHit | null>(null);
	detail = $state<ContentProjectDetail | null>(null);
	detailLoading = $state(false);
	detailError = $state<string | null>(null);
	private detailCache = new Map<string, { fetchedAt: number; detail: ContentProjectDetail }>();

	private searchToken = 0;
	private versionsToken = 0;
	private debounceTimer: ReturnType<typeof setTimeout> | undefined;

	constructor(getCtx: GetContext) {
		this.getCtx = getCtx;
	}

	get hasQuery() {
		return this.query.trim().length > 0;
	}

	get page() {
		return Math.floor(this.offset / this.limit) + 1;
	}

	get totalPages() {
		return Math.max(1, Math.ceil(this.totalHits / this.limit));
	}

	get hasNextPage() {
		return this.offset + this.limit < this.totalHits;
	}

	get hasPrevPage() {
		return this.offset > 0;
	}

	/** Lanza la búsqueda SIN dejar dependencias reactivas en quien llama.
	 *
	 * `executeSearch` lee `offset`/`query`/`selectedCategories`/`sortIndex`
	 * (y `instance`/`kind` vía `getCtx`) de forma síncrona, antes del
	 * primer `await`. Si eso corre dentro de un `$effect`/`$derived` (ej.
	 * el `$effect` de ContentManager.svelte que llama `resetForNewContext`,
	 * o el `$derived` de ContentExplorerView.svelte que llama `init`), el
	 * efecto quedaba suscrito a esos campos: tocar Siguiente cambiaba
	 * `offset`, el efecto se re-disparaba, `resetForNewContext` volvía
	 * `offset` a 0 y la paginación quedaba trabada en "Página 1 de X". Las
	 * dependencias que SÍ deben re-disparar una búsqueda las declara cada
	 * vista explícitamente (`instance.name`, `kind`, `subtab`). */
	private runSearch() {
		untrack(() => void this.executeSearch());
	}

	/** `null` cuando no hubo error — ver `errorKind` para distinguir cuál. */
	private async executeSearch() {
		const token = ++this.searchToken;
		this.loading = true;
		this.errorKind = null;
		this.errorMessage = null;
		try {
			const page = await dispatchSearch(
				this.getCtx(),
				this.query,
				[...this.selectedCategories],
				this.offset,
				this.sortIndex
			);
			if (token !== this.searchToken) return;
			this.results = page.hits;
			this.totalHits = page.total_hits;
		} catch (e) {
			if (token !== this.searchToken) return;
			const { kind, message } = classifyError(e);
			this.errorKind = kind;
			this.errorMessage = message;
			this.results = [];
			this.totalHits = 0;
		} finally {
			if (token === this.searchToken) this.loading = false;
		}
	}

	/** Debounce de 300ms, igual al que ya tenía ContentManager.svelte — se
	 * dispara igual con `query` vacío (así se ve el listado "populares" por
	 * defecto al entrar, sin que el usuario tenga que escribir nada; ver
	 * pedido del cliente de "exploración sin búsqueda"). */
	private scheduleSearch() {
		clearTimeout(this.debounceTimer);
		this.debounceTimer = setTimeout(() => this.runSearch(), 300);
	}

	/** Llamar una vez al montar la vista — trae el listado inicial sin que
	 * el usuario tenga que tocar nada. */
	init() {
		this.runSearch();
	}

	/** Instancia/kind cambiaron bajo la misma vista montada — vuelve a
	 * página 1 e invalida resultados viejos (pedido explícito del
	 * cliente). */
	resetForNewContext() {
		this.offset = 0;
		this.results = [];
		this.runSearch();
	}

	setQuery(q: string) {
		this.query = q;
		this.offset = 0;
		// Con query vacío, "Relevancia" no tiene sentido (regla del
		// cliente) — si el usuario borra todo el texto, se vuelve al
		// default de descubrimiento en vez de dejar un orden que ya no
		// aplica.
		if (!q.trim() && this.sortIndex === 'relevance') this.sortIndex = 'downloads';
		this.scheduleSearch();
	}

	toggleCategory(cat: string) {
		const next = new Set(this.selectedCategories);
		if (next.has(cat)) next.delete(cat);
		else next.add(cat);
		this.selectedCategories = next;
		this.offset = 0;
		this.scheduleSearch();
	}

	clearCategories() {
		if (this.selectedCategories.size === 0) return;
		this.selectedCategories = new Set();
		this.offset = 0;
		this.scheduleSearch();
	}

	setSort(index: SortIndex) {
		this.sortIndex = index;
		this.offset = 0;
		this.scheduleSearch();
	}

	goToNextPage() {
		if (!this.hasNextPage || this.loading) return;
		this.offset += this.limit;
		this.runSearch();
	}

	goToPrevPage() {
		if (!this.hasPrevPage || this.loading) return;
		this.offset = Math.max(0, this.offset - this.limit);
		this.runSearch();
	}

	/** `onInstalled` para que el caller refresque su propia lista de
	 * instalados (`ContentManager.svelte` ya tiene `refreshInstalled` —
	 * no se duplica esa lógica acá). */
	async install(projectId: string, versionId: string | undefined, onInstalled?: () => void) {
		this.installingId = projectId;
		this.installError = null;
		try {
			await dispatchInstall(this.getCtx(), projectId, versionId);
			this.openVersionsFor = null;
			onInstalled?.();
		} catch (e) {
			this.installError = String(e);
		} finally {
			this.installingId = null;
		}
	}

	/** Lista de versiones compatibles para la ficha de proyecto — separado
	 * de `toggleVersions` a propósito: ese método maneja el estado
	 * compartido `openVersionsFor`/`versionOptions` del dropdown inline de
	 * cada fila, y esto no debería pisarlo ni depender de él. */
	getVersionsFor(projectId: string): Promise<ModVersionSummary[]> {
		return dispatchVersions(this.getCtx(), projectId);
	}

	async toggleVersions(projectId: string) {
		if (this.openVersionsFor === projectId) {
			this.openVersionsFor = null;
			return;
		}
		this.openVersionsFor = projectId;
		this.versionOptions = [];
		this.loadingVersions = true;
		const token = ++this.versionsToken;
		try {
			const options = await dispatchVersions(this.getCtx(), projectId);
			if (token === this.versionsToken) this.versionOptions = options;
		} catch {
			if (token === this.versionsToken) this.versionOptions = [];
		} finally {
			if (token === this.versionsToken) this.loadingVersions = false;
		}
	}

	/** Abre la ficha de detalle — sub-vista interna, no desmonta nada de la
	 * búsqueda (query/filtros/orden/página quedan intactos para cuando se
	 * cierre). Caché de 10 min en memoria, misma duración que la del
	 * backend (ver `project_cache.rs`) — evita pedir de nuevo si se
	 * abre/cierra el mismo proyecto varias veces en la misma sesión.
	 *
	 * `hit` es opcional: si se tiene a mano el `ModSearchHit` del que se
	 * abrió la ficha (desde una fila de resultados), se guarda para poder
	 * mostrar su `author` — `GET /project/{id}` de Modrinth no lo expone
	 * (ver `ContentProjectDetail` en Rust). Sin `hit` (ej. se llega por
	 * otro camino) la ficha simplemente no muestra autor. */
	async openDetail(projectId: string, hit?: ModSearchHit) {
		this.detailProjectId = projectId;
		this.detailHit = hit ?? null;
		this.detailError = null;
		const cached = this.detailCache.get(projectId);
		if (cached && Date.now() - cached.fetchedAt < DETAIL_CACHE_TTL_MS) {
			this.detail = cached.detail;
			this.detailLoading = false;
			return;
		}
		this.detail = null;
		this.detailLoading = true;
		try {
			const d = await dispatchProjectDetail(this.getCtx(), projectId);
			this.detailCache.set(projectId, { fetchedAt: Date.now(), detail: d });
			if (this.detailProjectId === projectId) this.detail = d;
		} catch (e) {
			if (this.detailProjectId === projectId) this.detailError = String(e);
		} finally {
			if (this.detailProjectId === projectId) this.detailLoading = false;
		}
	}

	closeDetail() {
		this.detailProjectId = null;
		this.detailHit = null;
		this.detail = null;
		this.detailError = null;
	}
}

export function createContentExplorer(getCtx: GetContext): ContentExplorer {
	return new ContentExplorer(getCtx);
}

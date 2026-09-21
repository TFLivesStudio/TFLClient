// See https://svelte.dev/docs/kit/types#app.d.ts
// for information about these interfaces
declare global {
	namespace App {
		// interface Error {}
		// interface Locals {}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
	// Inyectada por vite.config.ts (define) desde package.json — versión
	// del build actual, para poder detectar "se acaba de actualizar".
	const __APP_VERSION__: string;
}

export {};

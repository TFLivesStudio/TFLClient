import { i18nState } from './index.svelte';
export const authText = (es: string, en: string) => (i18nState.locale === 'en' ? en : es);
const errors: Record<string, [string, string]> = {
	AUTH_CANCELLED: ['Inicio de sesión cancelado.', 'Sign-in cancelled.'],
	AUTH_TIMEOUT: [
		'La autorización caducó. Inicia un nuevo intento.',
		'Authorization expired. Start a new attempt.'
	],
	AUTH_DECLINED: [
		'No autorizaste el acceso. Puedes volver a intentarlo.',
		'Access was declined. You can try again.'
	],
	AUTH_BUSY: [
		'Ya hay un inicio de sesión en curso. Complétalo o cancélalo primero.',
		'Another sign-in is running. Finish or cancel it first.'
	],
	AUTH_FINISHING: ['La sesión está terminando de guardarse.', 'Your session is finishing saving.'],
	ACCOUNT_MISMATCH: [
		'Microsoft devolvió otra cuenta. Inicia sesión con la cuenta que quieres renovar.',
		'Microsoft returned another account. Sign in with the account you want to renew.'
	],
	REAUTHENTICATION_REQUIRED: [
		'Esta sesión necesita una nueva autorización de Microsoft.',
		'This session needs a new Microsoft authorization.'
	],
	SECURE_STORE_UNAVAILABLE: [
		'No se pudo guardar la sesión en el almacén seguro del sistema. Reintenta o elige una sesión temporal.',
		'The session could not be saved using the system secure store. Retry or choose a temporary session.'
	],
	LOCAL_STORAGE_ERROR: [
		'No se pudieron guardar los datos de la cuenta. Comprueba los permisos y el espacio disponible.',
		'Account data could not be saved. Check permissions and available disk space.'
	],
	BROWSER_UNAVAILABLE: [
		'No se pudo abrir el navegador. Usa el código de dispositivo.',
		'The browser could not be opened. Use a device code.'
	],
	STATE_MISMATCH: [
		'La respuesta no corresponde a este intento. Abre un inicio de sesión nuevo.',
		'The response does not belong to this attempt. Start a new sign-in.'
	],
	CALLBACK_INVALID: [
		'La respuesta de autorización no es válida. Vuelve a intentarlo.',
		'The authorization response is invalid. Try again.'
	],
	MICROSOFT_REGISTRATION_ERROR: [
		'Microsoft rechazó la configuración de la aplicación. Consulta al responsable del launcher.',
		'Microsoft rejected the application configuration. Contact the launcher maintainer.'
	],
	MICROSOFT_AUTH_ERROR: [
		'Microsoft no pudo autorizar la sesión. Reintenta o usa otro método.',
		'Microsoft could not authorize the session. Retry or use another method.'
	],
	MINECRAFT_APP_NOT_APPROVED: [
		'Minecraft rechazó el registro de esta aplicación (403 Invalid app registration). Su aprobación debe confirmarse; no cambies de cuenta para resolverlo.',
		'Minecraft rejected this application registration (403 Invalid app registration). Its approval must be confirmed; switching accounts will not fix it.'
	],
	MINECRAFT_SERVICES_ERROR: [
		'Minecraft Services no pudo iniciar la sesión. Reintenta más tarde.',
		'Minecraft Services could not sign you in. Try again later.'
	],
	MINECRAFT_PROFILE_REQUIRED: [
		'La cuenta no tiene un perfil de Minecraft Java disponible. Comprueba tu licencia y crea el perfil en minecraft.net.',
		'This account has no available Minecraft Java profile. Check your license and create the profile at minecraft.net.'
	],
	MINECRAFT_PROFILE_INVALID: [
		'Minecraft devolvió un perfil no válido. No se guardó la cuenta.',
		'Minecraft returned an invalid profile. The account was not saved.'
	],
	MINECRAFT_PROFILE_ERROR: [
		'No se pudo consultar el perfil de Minecraft. Vuelve a intentarlo.',
		'The Minecraft profile could not be retrieved. Try again.'
	],
	XBOX_PROFILE_REQUIRED: [
		'Esta cuenta necesita un perfil de Xbox. Créalo con Microsoft y vuelve a intentarlo.',
		'This account needs an Xbox profile. Create one with Microsoft and try again.'
	],
	XBOX_FAMILY_RESTRICTION: [
		'Xbox requiere revisar la configuración familiar de esta cuenta.',
		'Xbox requires a review of this account’s family settings.'
	],
	XBOX_LIVE_ERROR: [
		'Xbox Live no pudo verificar esta cuenta.',
		'Xbox Live could not verify this account.'
	],
	XBOX_XSTS_ERROR: [
		'Xbox no autorizó el acceso a Minecraft. Revisa las restricciones de la cuenta.',
		'Xbox did not authorize access to Minecraft. Review account restrictions.'
	],
	NETWORK_ERROR: [
		'No se pudo conectar. Comprueba tu conexión y vuelve a intentarlo.',
		'Connection failed. Check your network and try again.'
	],
	AUTH_RATE_LIMITED: [
		'El servicio está limitando solicitudes. Espera y vuelve a intentarlo.',
		'The service is limiting requests. Wait and try again.'
	],
	AUTH_SERVICE_UNAVAILABLE: [
		'El servicio de autenticación no está disponible temporalmente.',
		'The authentication service is temporarily unavailable.'
	]
};
export function authError(code: string | null) {
	const pair = code ? errors[code] : null;
	return pair
		? authText(...pair)
		: authText(
				'No se pudo completar el inicio de sesión. Reintenta o elige otro método.',
				'Sign-in could not be completed. Retry or choose another method.'
			);
}
export function authPhase(phase: string) {
	const phases: Record<string, [string, string]> = {
		preparing: ['Preparando inicio de sesión', 'Preparing sign-in'],
		opening: ['Abriendo Microsoft', 'Opening Microsoft'],
		waiting: ['Esperando tu autorización', 'Waiting for your authorization'],
		microsoft: ['Verificando Microsoft', 'Verifying Microsoft'],
		xbox: ['Verificando Xbox Live', 'Verifying Xbox Live'],
		xsts: ['Comprobando permisos de Xbox', 'Checking Xbox permissions'],
		minecraft: ['Conectando con Minecraft', 'Connecting to Minecraft'],
		profile: ['Verificando tu perfil', 'Verifying your profile'],
		saving: ['Guardando la sesión', 'Saving your session'],
		storage_required: ['Elige cómo conservar esta sesión', 'Choose how to keep this session'],
		success: ['Sesión iniciada', 'Signed in'],
		cancelled: ['Inicio de sesión cancelado', 'Sign-in cancelled'],
		expired: ['La autorización caducó', 'Authorization expired'],
		error: ['No se pudo iniciar sesión', 'Sign-in failed'],
		external_fallback: ['Continuando en tu navegador', 'Continuing in your browser'],
		device_fallback: ['Preparando un código de dispositivo', 'Preparing a device code']
	};
	return authText(...(phases[phase] || phases.preparing));
}

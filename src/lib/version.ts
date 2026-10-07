// Las versiones beta son prerelease de semver (`0.11.0-beta.1`). Para la gente
// se muestran como "0.11.0 (beta)" — el número solo si hay más de una beta de
// la misma versión ("0.11.0 (beta 2)"). Cualquier otro sufijo se muestra tal cual.

const PRERELEASE = /^(\d+\.\d+\.\d+)-([0-9A-Za-z.-]+)$/;

export function formatVersion(version: string): string {
	const match = PRERELEASE.exec(version);
	if (!match) return version;
	const [, base, pre] = match;
	const beta = /^beta(?:\.(\d+))?$/.exec(pre);
	if (beta) return beta[1] && beta[1] !== '1' ? `${base} (beta ${beta[1]})` : `${base} (beta)`;
	return `${base} (${pre.replace('.', ' ')})`;
}

export function isPrereleaseVersion(version: string): boolean {
	return PRERELEASE.test(version);
}

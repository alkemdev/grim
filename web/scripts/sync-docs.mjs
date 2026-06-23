// Generate the Starlight docs content from the repo's canonical docs/ tree.
//
// docs/ is the single source of truth. This copies its markdown into src/content/docs/, adding the
// `title` frontmatter Starlight needs (from each file's H1), mapping README.md -> index.md, and
// rewriting internal `*.md` links to extensionless routes. The generated files are git-ignored; run
// this (it runs automatically before `dev` and `build`) to refresh them.

import { readdir, readFile, writeFile, mkdir, rm, stat } from 'node:fs/promises';
import { dirname, join, relative, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';

const here = dirname(fileURLToPath(import.meta.url));
const DOCS = resolve(here, '../../docs');
const OUT = resolve(here, '../src/content/docs');

// Generated outputs, cleaned before each run so deletions in docs/ propagate.
const GENERATED_DIRS = ['concepts', 'decisions'];
const GENERATED_FILES = ['architecture.md', 'roadmap.md'];

async function walk(dir) {
	const out = [];
	for (const entry of await readdir(dir, { withFileTypes: true })) {
		const full = join(dir, entry.name);
		if (entry.isDirectory()) out.push(...(await walk(full)));
		else if (entry.name.endsWith('.md')) out.push(full);
	}
	return out;
}

function titleFrom(content, fallback) {
	const match = content.match(/^#\s+(.+)$/m);
	return match ? match[1].trim() : fallback;
}

function stripFirstH1(content) {
	return content.replace(/^#\s+.+\n+/m, '');
}

function rewriteLinks(content) {
	return content.replace(/\]\(([^)]+)\)/g, (whole, target) => {
		if (/^[a-z]+:\/\//i.test(target) || target.startsWith('#') || target.startsWith('mailto:')) {
			return whole;
		}
		let [path, anchor] = target.split('#');
		path = path.replace(/\.md$/i, '');
		path = path.replace(/(^|\/)README$/i, '$1');
		if (path === '') path = '.';
		return `](${path}${anchor ? '#' + anchor : ''})`;
	});
}

async function main() {
	for (const dir of GENERATED_DIRS) await rm(join(OUT, dir), { recursive: true, force: true });
	for (const file of GENERATED_FILES) await rm(join(OUT, file), { force: true });

	const files = await walk(DOCS);
	let count = 0;
	for (const file of files) {
		const rel = relative(DOCS, file);
		if (rel === 'README.md') continue; // the landing page covers the docs index
		const raw = await readFile(file, 'utf8');
		const title = titleFrom(raw, rel.replace(/\.md$/, ''));
		const body = rewriteLinks(stripFirstH1(raw));
		const frontmatter = `---\ntitle: ${JSON.stringify(title)}\n---\n\n`;
		const destRel = rel.replace(/(^|\/)README\.md$/i, '$1index.md');
		const dest = join(OUT, destRel);
		await mkdir(dirname(dest), { recursive: true });
		await writeFile(dest, frontmatter + body);
		count++;
	}
	console.log(`sync-docs: wrote ${count} page(s) from ${relative(resolve(here, '..'), DOCS)}/`);
}

await stat(DOCS).catch(() => {
	throw new Error(`docs source not found at ${DOCS}`);
});
await main();

// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';

// https://astro.build/config
export default defineConfig({
	site: 'https://grim.alkem.dev',
	integrations: [
		starlight({
			title: 'grim',
			tagline: 'A grimoire for your machines.',
			social: [
				{ icon: 'github', label: 'GitHub', href: 'https://github.com/alkemdev/grim' },
			],
			editLink: {
				baseUrl: 'https://github.com/alkemdev/grim/edit/main/docs/',
			},
			customCss: ['./src/styles/grim.css'],
			// Content under concepts/ and decisions/ is generated from the repo's docs/ by
			// scripts/sync-docs.mjs — docs/ is the single source of truth.
			sidebar: [
				{
					label: 'Start here',
					items: [
						{ label: 'Introduction', link: '/' },
						{ label: 'Architecture', link: '/architecture/' },
						{ label: 'Roadmap', link: '/roadmap/' },
					],
				},
				{ label: 'Concepts', items: [{ autogenerate: { directory: 'concepts' } }] },
				{ label: 'Decisions', items: [{ autogenerate: { directory: 'decisions' } }] },
			],
		}),
	],
});

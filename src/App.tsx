import { Button } from "@heroui/react";

const stack = [
	"Tauri 2",
	"React 19",
	"TypeScript",
	"Biome",
	"HeroUI 3",
	"Tailwind CSS 4",
];

function App() {
	return (
		<main className="min-h-screen bg-linear-to-br from-zinc-950 via-slate-950 to-indigo-950 px-6 py-16 text-white">
			<section className="mx-auto flex min-h-[calc(100vh-8rem)] max-w-5xl flex-col justify-center">
				<div className="mb-8 inline-flex w-fit items-center gap-2 rounded-full border border-white/10 bg-white/5 px-3 py-1 text-sm text-indigo-200 backdrop-blur">
					<span className="size-2 rounded-full bg-emerald-400 shadow-[0_0_12px_theme(colors.emerald.400)]" />
					Scaffold ready
				</div>

				<p className="mb-3 font-medium text-indigo-300">
					Local AI agent performance monitor
				</p>
				<h1 className="max-w-3xl text-balance text-5xl font-semibold tracking-tight sm:text-7xl">
					Measure where your agents spend their time.
				</h1>
				<p className="mt-6 max-w-2xl text-pretty text-lg leading-8 text-slate-300">
					Agent Speed will compare local Codex, Claude Code, and other agent
					runs with one consistent timeline and metric model.
				</p>

				<div className="mt-10 flex flex-wrap items-center gap-3">
					<Button variant="tertiary">Start building</Button>
					<span className="text-sm text-slate-400">
						Edit src/App.tsx to continue
					</span>
				</div>

				<div className="mt-16 grid gap-3 sm:grid-cols-2 lg:grid-cols-3">
					{stack.map((item) => (
						<div
							className="rounded-2xl border border-white/10 bg-white/5 px-5 py-4 text-sm text-slate-200 backdrop-blur"
							key={item}
						>
							{item}
						</div>
					))}
				</div>
			</section>
		</main>
	);
}

export default App;

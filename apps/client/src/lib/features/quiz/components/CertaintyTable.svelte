<script lang="ts">
	import { Field } from "@formisch/svelte"

	let { form } = $props<{ form: any }>()
</script>

<div class="panel-muted mt-6 space-y-3 p-4 sm:p-5">
	<p class="section-kicker m-0 mb-1">Parametros de certeza</p>
	<p class="m-0 text-sm leading-relaxed text-zinc-700">
		Define el puntaje por nivel de certeza para respuestas correctas e incorrectas.
	</p>
	<div class="overflow-x-auto">
		<table class="w-full table-fixed border-collapse text-sm">
			<colgroup>
				<col class="w-[22%]" />
				<col class="w-[39%]" />
				<col class="w-[39%]" />
			</colgroup>
			<thead>
				<tr class="text-left text-zinc-700">
					<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
						>Nivel</th
					>
					<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
						>Correcta</th
					>
					<th class="border border-zinc-300 bg-white p-2 text-xs font-semibold"
						>Incorrecta</th
					>
				</tr>
			</thead>
			<tbody>
				<tr>
					<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
						>Baja</td
					>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "low", "correct"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "low", "incorrect"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
				</tr>
				<tr>
					<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
						>Media</td
					>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "medium", "correct"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "medium", "incorrect"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
				</tr>
				<tr>
					<td class="border border-zinc-300 bg-white p-2 text-sm text-zinc-800"
						>Alta</td
					>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "high", "correct"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
					<td class="border border-zinc-300 bg-white p-2">
						<Field of={form} path={["certainty", "high", "incorrect"] as any}>
							{#snippet children(field)}
								<input
									{...field.props}
									class="input-base h-10 min-h-10 px-2.5 py-2 text-sm"
									type="number"
									step="1"
									inputmode="numeric"
									value={field.input}
								/>
							{/snippet}
						</Field>
					</td>
				</tr>
			</tbody>
		</table>
	</div>
</div>

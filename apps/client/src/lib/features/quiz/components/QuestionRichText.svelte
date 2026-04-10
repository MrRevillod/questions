<script lang="ts">
	import DOMPurify from "dompurify"
	import katex from "katex"
	import "katex/dist/katex.min.css"

	type Segment =
		| {
				kind: "html"
				content: string
		  }
		| {
				kind: "math"
				content: string
		  }

	const ALLOWED_TAGS = [
		"b",
		"strong",
		"i",
		"em",
		"u",
		"br",
		"p",
		"ul",
		"ol",
		"li",
		"sub",
		"sup",
		"span",
	]
	const INLINE_LATEX_PATTERN = /(\\\((?:.|\n|\r)*?\\\))/g

	let { text = "" }: { text?: string } = $props()

	const sanitizeHtml = (value: string) => {
		return DOMPurify.sanitize(value, {
			ALLOWED_TAGS,
			ALLOWED_ATTR: [],
		})
	}

	const splitIntoSegments = (value: string): Segment[] => {
		if (!value) {
			return []
		}

		const segments: Segment[] = []
		let cursor = 0

		for (const match of value.matchAll(INLINE_LATEX_PATTERN)) {
			const fullMatch = match[0]
			const startIndex = match.index ?? 0

			if (startIndex > cursor) {
				segments.push({
					kind: "html",
					content: value.slice(cursor, startIndex),
				})
			}

			segments.push({
				kind: "math",
				content: fullMatch.slice(2, -2).trim(),
			})

			cursor = startIndex + fullMatch.length
		}

		if (cursor < value.length) {
			segments.push({
				kind: "html",
				content: value.slice(cursor),
			})
		}

		return segments
	}

	const renderedSegments = $derived.by(() => {
		return splitIntoSegments(text).map(segment => {
			if (segment.kind === "html") {
				return {
					kind: "html" as const,
					content: sanitizeHtml(segment.content),
				}
			}

			try {
				return {
					kind: "math" as const,
					content: katex.renderToString(segment.content, {
						throwOnError: true,
						displayMode: false,
						strict: "warn",
					}),
				}
			} catch {
				return {
					kind: "html" as const,
					content: sanitizeHtml(`\\(${segment.content}\\)`),
				}
			}
		})
	})
</script>

<div class="question-rich-text leading-relaxed text-pretty">
	{#if renderedSegments.length === 0}
		<span></span>
	{:else}
		{#each renderedSegments as segment, index (`${segment.kind}-${index}`)}
			{@html segment.content}
		{/each}
	{/if}
</div>

<style>
	.question-rich-text :global(p) {
		margin: 0;
	}

	.question-rich-text :global(ul),
	.question-rich-text :global(ol) {
		margin: 0;
		padding-left: 1.3rem;
	}

	.question-rich-text :global(li) {
		margin: 0;
	}
</style>

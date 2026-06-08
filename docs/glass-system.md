# GlassOS Design System: OpenSeason Developer Guide

OpenSeason implements an advanced, high-performance glass refraction system (**GlassOS**) that moves beyond basic passive blurs to true interactive light refraction.

---

## 1. Quick Start: Using `GlassOSPanel`

To turn any layout component or card into a refracting glass panel, import and wrap it in the Svelte `GlassOSPanel` component:

```html
<script lang="ts">
  import GlassOSPanel from "$lib/components/GlassOSPanel.svelte";
</script>

<GlassOSPanel borderRadius={8} scale={15} depth={20} enableSpecular={true}>
  <div class="p-6">
    <h3 class="text-lg font-bold">My Premium Widget</h3>
    <p class="text-sm text-muted-foreground">Content here...</p>
  </div>
</GlassOSPanel>
```

---

## 2. Properties & Configuration

The `<GlassOSPanel>` accepts the following props:

| Prop | Type | Default | Description |
| :--- | :--- | :--- | :--- |
| `children` | `Snippet` | *Required* | Svelte 5 snippet representing inner content. |
| `borderRadius` | `number` | `16` | Corner radius of the panel. |
| `scale` | `number` | `20` | Refraction/bending strength. |
| `depth` | `number` | `24` | Transition zone depth (thickness of border refraction). |
| `enableSpecular` | `boolean` | `true` | Enables real-time cursor light highlight tracking. |
| `className` | `string` | `""` | Additional Tailwind/CSS classes applied to the outer container. |
| `style` | `string` | `""` | Inline styles applied to the outer container. |

---

## 3. How It Works Under the Hood

The panel uses a dynamic canvas to build a displacement map corresponding to a 3D lens.
1. **Symmetry Optimization**: Only the top-left quadrant of the lens is calculated. This is mirrored into the other three quadrants (reversing displacement vectors accordingly) to reduce per-frame pixel math by 75%, maintaining 60 FPS.
2. **Safari Invalidation**: WebKit caches SVG filters by ID. To prevent frozen animations, the component generates a unique filter ID (`glass-filter-[uuid]-[timestamp]`) on every layout resize or change.
3. **Cursor Glare**: Feeds client cursor coordinates $(X, Y)$ into the `<fePointLight>` filter primitive to slide specular reflections across the card face.

---

## 4. CSS Classes & Customization

The system defines several global utility selectors in `app.css`:
*   `.glass-os-panel-container`: Prepares layout, handles transform offsets on hover.
*   `.glass-os-specular-glow`: Adds the radial glare highlight.
*   `.glass-os-border-glare`: Draws the inner card border and applies the fallback backdrop blur.
*   `.glass-os-refracted-content`: Prepares the z-index layer for filtered elements.

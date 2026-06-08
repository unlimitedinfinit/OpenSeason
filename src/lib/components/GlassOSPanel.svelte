<script lang="ts">
  import { onMount } from "svelte";

  let {
    children,
    className = "",
    borderRadius = 16,
    scale = 20,
    depth = 24,
    enableSpecular = true,
    style = ""
  } = $props();

  let containerEl = $state<HTMLDivElement | null>(null);
  let dimensions = $state({ width: 0, height: 0 });
  let mapDataUrl = $state("");
  let filterId = $state("");
  let lightPos = $state({ x: 50, y: 50, z: 100 });
  let isHovered = $state(false);
  const baseId = Math.random().toString(36).substring(2, 9);

  // Triggered when dimensions change
  function generateMap(width: number, height: number) {
    if (width === 0 || height === 0) return;

    const canvas = document.createElement("canvas");
    canvas.width = width;
    canvas.height = height;
    const ctx = canvas.getContext("2d");
    if (!ctx) return;

    const imgData = ctx.createImageData(width, height);
    const data = imgData.data;

    const R = borderRadius;
    const D = depth;
    const maxRefract = 127;

    const midX = Math.ceil(width / 2);
    const midY = Math.ceil(height / 2);

    for (let y = 0; y < midY; y++) {
      for (let x = 0; x < midX; x++) {
        let dx = 0;
        let dy = 0;

        if (x < R && y < R) {
          const cx = R;
          const cy = R;
          const distFromCornerCenter = Math.sqrt((x - cx) ** 2 + (y - cy) ** 2);
          const distToBorder = R - distFromCornerCenter;

          if (distFromCornerCenter < R) {
            if (distToBorder < D && distFromCornerCenter > 0) {
              const strength = 1 - distToBorder / D;
              const angleX = (x - cx) / distFromCornerCenter;
              const angleY = (y - cy) / distFromCornerCenter;
              dx = angleX * strength * maxRefract;
              dy = angleY * strength * maxRefract;
            }
          }
        } else {
          if (x < D) {
            dx = -maxRefract * (1 - x / D);
          }
          if (y < D) {
            dy = -maxRefract * (1 - y / D);
          }
        }

        const quadrants = [
          { targetX: x, targetY: y, multX: 1, multY: 1 },
          { targetX: width - 1 - x, targetY: y, multX: -1, multY: 1 },
          { targetX: x, targetY: height - 1 - y, multX: 1, multY: -1 },
          { targetX: width - 1 - x, targetY: height - 1 - y, multX: -1, multY: -1 },
        ];

        quadrants.forEach(({ targetX, targetY, multX, multY }) => {
          if (targetX >= 0 && targetX < width && targetY >= 0 && targetY < height) {
            const idx = (targetY * width + targetX) * 4;
            data[idx] = Math.round(127 + dx * multX);
            data[idx + 1] = Math.round(127 + dy * multY);
            data[idx + 2] = 0;
            data[idx + 3] = 255;
          }
        });
      }
    }

    ctx.putImageData(imgData, 0, 0);
    mapDataUrl = canvas.toDataURL("image/png");
    filterId = `glass-filter-${baseId}-${Date.now()}`;
  }

  onMount(() => {
    if (!containerEl) return;

    const updateSize = () => {
      if (containerEl) {
        dimensions = {
          width: containerEl.offsetWidth,
          height: containerEl.offsetHeight
        };
        generateMap(dimensions.width, dimensions.height);
      }
    };

    updateSize();

    const observer = new ResizeObserver(updateSize);
    observer.observe(containerEl);

    // Mouse movement specular glare tracking
    const handleMouseMove = (e: MouseEvent) => {
      if (!enableSpecular || !containerEl) return;
      const rect = containerEl.getBoundingClientRect();
      lightPos = {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
        z: 120
      };
      isHovered = true;
    };

    const handleMouseLeave = () => {
      isHovered = false;
    };

    containerEl.addEventListener("mousemove", handleMouseMove);
    containerEl.addEventListener("mouseleave", handleMouseLeave);

    return () => {
      observer.disconnect();
      if (containerEl) {
        containerEl.removeEventListener("mousemove", handleMouseMove);
        containerEl.removeEventListener("mouseleave", handleMouseLeave);
      }
    };
  });
</script>

<div
  bind:this={containerEl}
  class="glass-os-panel-container {className}"
  style="border-radius: {borderRadius}px; {style}"
>
  {#if mapDataUrl && filterId}
    <svg
      style="position: absolute; width: 0; height: 0; pointer-events: none;"
      aria-hidden="true"
    >
      <defs>
        <filter id={filterId} color-interpolation-filters="sRGB" x="-20%" y="-20%" width="140%" height="140%">
          <feImage href={mapDataUrl} result="displacementMap" />
          <feDisplacementMap
            in="SourceGraphic"
            in2="displacementMap"
            scale={scale}
            xChannelSelector="R"
            yChannelSelector="G"
            result="refracted"
          />
          
          {#if enableSpecular}
            <feSpecularLighting
              in="displacementMap"
              specularExponent="120"
              specularConstant={isHovered ? 0.35 : 0}
              lighting-color="#ffd54f"
              result="specularLight"
            >
              <fePointLight x={lightPos.x} y={lightPos.y} z={lightPos.z} />
            </feSpecularLighting>
            
            <feComposite
              in="specularLight"
              in2="SourceGraphic"
              operator="in"
              result="maskedSpecular"
            />
            
            <feBlend
              in="maskedSpecular"
              in2="refracted"
              mode="screen"
            />
          {/if}
        </filter>
      </defs>
    </svg>
  {/if}

  <div
    class="glass-os-refracted-content"
    style="filter: {filterId ? `url(#${filterId})` : 'none'}; border-radius: inherit; width: 100%; height: 100%;"
  >
    {@render children?.()}
  </div>

  <div
    class="glass-os-specular-glow"
    style="border-radius: {borderRadius}px"
  ></div>
  <div
    class="glass-os-border-glare"
    style="border-radius: {borderRadius}px"
  ></div>
</div>

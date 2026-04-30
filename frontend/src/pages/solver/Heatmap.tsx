import { useEffect, useRef } from 'react'

interface HeatmapProps {
  /** 2D array: data[x][y] */
  data: number[][]
  width?: number
  height?: number
  colormap?: 'viridis' | 'plasma' | 'coolwarm'
  label?: string
}

// Simple colormaps (sampled from matplotlib)
function viridis(t: number): [number, number, number] {
  // t in [0,1]
  const stops: [number, [number, number, number]][] = [
    [0.0, [68, 1, 84]],
    [0.25, [59, 82, 139]],
    [0.5, [33, 145, 140]],
    [0.75, [94, 201, 98]],
    [1.0, [253, 231, 37]],
  ]
  return interpolateColormap(t, stops)
}

function plasma(t: number): [number, number, number] {
  const stops: [number, [number, number, number]][] = [
    [0.0, [13, 8, 135]],
    [0.25, [126, 3, 168]],
    [0.5, [204, 71, 120]],
    [0.75, [248, 149, 64]],
    [1.0, [240, 249, 33]],
  ]
  return interpolateColormap(t, stops)
}

function coolwarm(t: number): [number, number, number] {
  const stops: [number, [number, number, number]][] = [
    [0.0, [59, 76, 192]],
    [0.25, [144, 178, 254]],
    [0.5, [221, 221, 221]],
    [0.75, [240, 158, 116]],
    [1.0, [180, 4, 38]],
  ]
  return interpolateColormap(t, stops)
}

function interpolateColormap(t: number, stops: [number, [number, number, number]][]): [number, number, number] {
  t = Math.max(0, Math.min(1, t))
  for (let i = 0; i < stops.length - 1; i++) {
    const [t0, c0] = stops[i]
    const [t1, c1] = stops[i + 1]
    if (t >= t0 && t <= t1) {
      const u = (t - t0) / (t1 - t0)
      return [
        Math.round(c0[0] + u * (c1[0] - c0[0])),
        Math.round(c0[1] + u * (c1[1] - c0[1])),
        Math.round(c0[2] + u * (c1[2] - c0[2])),
      ]
    }
  }
  return stops[stops.length - 1][1]
}

export function Heatmap({ data, width = 320, height = 320, colormap = 'viridis', label }: HeatmapProps) {
  const canvasRef = useRef<HTMLCanvasElement>(null)

  useEffect(() => {
    const canvas = canvasRef.current
    if (!canvas || data.length === 0) return
    const ctx = canvas.getContext('2d')
    if (!ctx) return

    const nx = data.length
    const ny = data[0]?.length ?? 0
    if (nx === 0 || ny === 0) return

    // Find min/max for normalization
    let min = Infinity
    let max = -Infinity
    for (let i = 0; i < nx; i++) {
      for (let j = 0; j < ny; j++) {
        const v = data[i][j]
        if (v < min) min = v
        if (v > max) max = v
      }
    }
    const range = max - min || 1

    // Draw pixel by pixel
    const imgData = ctx.createImageData(width, height)
    const colorFn = colormap === 'plasma' ? plasma : colormap === 'coolwarm' ? coolwarm : viridis

    for (let px = 0; px < width; px++) {
      for (let py = 0; py < height; py++) {
        // Map pixel to data index
        const xi = Math.min(nx - 1, Math.floor((px / width) * nx))
        const yi = Math.min(ny - 1, Math.floor((py / height) * ny))
        const t = (data[xi][yi] - min) / range
        const [r, g, b] = colorFn(t)
        const idx = (py * width + px) * 4
        imgData.data[idx] = r
        imgData.data[idx + 1] = g
        imgData.data[idx + 2] = b
        imgData.data[idx + 3] = 255
      }
    }
    ctx.putImageData(imgData, 0, 0)

    // Draw colorbar legend values
    ctx.font = '11px monospace'
    ctx.fillStyle = 'rgba(0,0,0,0.7)'
    ctx.fillText(max.toExponential(2), 4, 12)
    ctx.fillText(min.toExponential(2), 4, height - 4)
  }, [data, width, height, colormap])

  return (
    <div className="inline-block">
      {label && <p className="text-xs text-center text-muted-foreground mb-1 font-mono">{label}</p>}
      <canvas ref={canvasRef} width={width} height={height} className="rounded border border-border" />
    </div>
  )
}

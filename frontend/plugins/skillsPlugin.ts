/**
 * Vite plugin: serves the `skills/` directory via the dev server.
 *
 * Routes:
 *   GET /api/skills/list                → { skills: string[] }           list skill packages
 *   GET /api/skills/:pkg/files          → { files: string[] }            list files in a package
 *   GET /api/skills/:pkg/file?f=name    → text/plain                     read one file
 *   GET /api/skills/:pkg/download       → application/zip                download package as zip
 */

import type { Plugin, ResolvedConfig } from 'vite'
import fs from 'fs'
import path from 'path'
import { IncomingMessage, ServerResponse } from 'http'
import archiver from 'archiver'

function send(res: ServerResponse, status: number, body: unknown, contentType = 'application/json') {
  const data = contentType === 'application/json' ? JSON.stringify(body) : (body as string)
  res.writeHead(status, { 'Content-Type': contentType, 'Access-Control-Allow-Origin': '*' })
  res.end(data)
}

function safeJoin(base: string, ...parts: string[]): string | null {
  const joined = path.resolve(base, ...parts)
  return joined.startsWith(base) ? joined : null
}

export function skillsPlugin(): Plugin {
  let resolvedConfig: ResolvedConfig

  return {
    name: 'vite-plugin-skills',
    configResolved(cfg) {
      resolvedConfig = cfg
    },
    configureServer(server) {
      server.middlewares.use(async (req: IncomingMessage, res: ServerResponse, next) => {
        const url = req.url ?? ''
        if (!url.startsWith('/api/skills')) return next()

        // Skills directory lives at <project-root>/skills
        const SKILLS_DIR = path.resolve(resolvedConfig.root, 'skills')

        // ── GET /api/skills/list ──────────────────────────────────────────
        if (url === '/api/skills/list') {
          if (!fs.existsSync(SKILLS_DIR)) return send(res, 200, { skills: [] })
          const entries = fs.readdirSync(SKILLS_DIR, { withFileTypes: true })
          const skills = entries.filter(e => e.isDirectory()).map(e => e.name)
          return send(res, 200, { skills })
        }

        // ── /api/skills/:pkg/* ────────────────────────────────────────────
        const match = url.match(/^\/api\/skills\/([^/?]+)(\/[^?]*)?(\?.*)?$/)
        if (!match) return next()
        const [, pkg, sub] = match
        const pkgDir = safeJoin(SKILLS_DIR, pkg)
        if (!pkgDir || !fs.existsSync(pkgDir)) return send(res, 404, { error: 'skill not found' })

        // GET /api/skills/:pkg/files
        if (sub === '/files') {
          const entries = fs.readdirSync(pkgDir, { withFileTypes: true })
          const files = entries.filter(e => e.isFile()).map(e => e.name)
          return send(res, 200, { files })
        }

        // GET /api/skills/:pkg/file?f=filename
        if (sub === '/file') {
          const params = new URLSearchParams(url.split('?')[1] ?? '')
          const filename = params.get('f')
          if (!filename) return send(res, 400, { error: 'missing ?f= param' })
          const filePath = safeJoin(pkgDir, filename)
          if (!filePath || !fs.existsSync(filePath)) return send(res, 404, { error: 'file not found' })
          const content = fs.readFileSync(filePath, 'utf-8')
          return send(res, 200, content, 'text/plain; charset=utf-8')
        }

        // GET /api/skills/:pkg/download
        if (sub === '/download') {
          res.writeHead(200, {
            'Content-Type': 'application/zip',
            'Content-Disposition': `attachment; filename="${pkg}.zip"`,
            'Access-Control-Allow-Origin': '*',
          })
          const archive = archiver('zip', { zlib: { level: 9 } })
          archive.pipe(res)
          archive.directory(pkgDir, pkg)
          await archive.finalize()
          return
        }

        next()
      })
    },
  }
}

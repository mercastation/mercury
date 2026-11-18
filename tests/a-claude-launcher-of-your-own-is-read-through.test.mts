// A LAUNCHER OF YOUR OWN IN FRONT OF CLAUDE CODE STOPPED THE APP FROM OPENING.
//
// Measured 2026-11-18: a CLI installer put ~/.local/bin at the front of the
// login shell's PATH, and the `claude` there was a 96-byte `exec python3
// launcher.py "$@"` that runs a setup step and then execs the real binary. The
// shell named the launcher, mise was not installed to follow it, so the build
// read the launcher's two lines, found none of the commands, and `npm start`
// stopped before the window. The real binary was one entry further down the
// list the search had just walked.
import { describe, it, expect } from 'vitest';
import fs from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { commandBundle, installedBundle } from '../scripts/read-claude-commands.mjs';

const noMise = () => null;

function place() {
  const dir = fs.mkdtempSync(path.join(os.tmpdir(), 'claude-launcher-'));
  const write = (name: string, body: string) => {
    const file = path.join(dir, name);
    fs.mkdirSync(path.dirname(file), { recursive: true });
    fs.writeFileSync(file, body);
    return file;
  };
  const launcher = write('own/claude', '#!/bin/sh\nexec /usr/bin/python3 /somewhere/launch.py -- "$@"\n');
  const bundle = write('brew/claude', 'not a script');
  return { dir, write, launcher, bundle, done: () => fs.rmSync(dir, { recursive: true, force: true }) };
}

describe('a launcher nothing can follow is read through to the Claude Code beside it', () => {
  it('reads the first other copy that is a real file and not a script', () => {
    const p = place();
    try {
      const other = p.write('other/claude', '#!/bin/bash\nexec claude "$@"\n');
      fs.mkdirSync(path.join(p.dir, 'folder/claude'), { recursive: true });
      const others = [p.launcher, path.join(p.dir, 'missing/claude'), path.join(p.dir, 'folder/claude'), other, p.bundle];
      expect(commandBundle(p.launcher, { which: noMise, others })).toBe(fs.realpathSync(p.bundle));
    } finally { p.done(); }
  });

  it('follows a symlink to the bundle, as Homebrew installs it', () => {
    const p = place();
    try {
      const link = path.join(p.dir, 'bin/claude');
      fs.mkdirSync(path.dirname(link), { recursive: true });
      fs.symlinkSync(p.bundle, link);
      expect(commandBundle(p.launcher, { which: noMise, others: [link] })).toBe(fs.realpathSync(p.bundle));
    } finally { p.done(); }
  });

  it('keeps the launcher when there is no other copy, so the build names what it read', () => {
    const p = place();
    try {
      expect(commandBundle(p.launcher, { which: noMise, others: [p.launcher] })).toBe(fs.realpathSync(p.launcher));
      expect(commandBundle(p.launcher, { which: noMise })).toBe(fs.realpathSync(p.launcher));
    } finally { p.done(); }
  });

  it('still takes the binary mise names over the list', () => {
    const p = place();
    try {
      const mised = p.write('mise/claude', 'not a script either');
      expect(commandBundle(p.launcher, { which: () => mised, others: [p.bundle] })).toBe(fs.realpathSync(mised));
    } finally { p.done(); }
  });

  it('never looks past a real binary', () => {
    const p = place();
    try {
      const elsewhere = p.write('elsewhere/claude', 'another real one');
      expect(commandBundle(p.bundle, { which: noMise, others: [elsewhere] })).toBe(fs.realpathSync(p.bundle));
    } finally { p.done(); }
  });

  it('hands the search its own list, and has no bundle when Claude Code is not found', () => {
    const p = place();
    try {
      const found = { path: p.launcher, found: true, certain: true, from: 'shell', searched: [p.launcher, p.bundle], evidence: null };
      expect(installedBundle(found, { which: noMise })).toBe(fs.realpathSync(p.bundle));
      expect(installedBundle({ ...found, path: null, found: false }, { which: noMise })).toBe(null);
    } finally { p.done(); }
  });
});

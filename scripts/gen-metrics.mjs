/**
 * Generates packages/render/src/style/metrics.rs — the Adobe AFM advance widths
 * and kern pairs for the standard Helvetica faces.
 *
 * The tables are read from @react-pdf/pdfkit so that measurement here matches the
 * reference renderer exactly, kerning included. Widths drive line breaking, so a
 * systematic difference would shift every wrap point in the document.
 *
 * Run it from a checkout of the reference repo, which has the dependency:
 *
 *   node ../cv_rs/scripts/gen-metrics.mjs ../cv_rs/packages/render/src/style/metrics.rs
 *
 * Only needed if the reference changes font.
 */
import PDFDocumentNS from '@react-pdf/pdfkit';
import fs from 'node:fs';

const PDFDocument = PDFDocumentNS.default ?? PDFDocumentNS;
const doc = new PDFDocument({ size: 'A4' });

/** WinAnsiEncoding: byte -> Unicode code point. */
const winansi = new Array(256).fill(null);
for (let b = 0x20; b <= 0x7e; b++) winansi[b] = b;
for (let b = 0xa0; b <= 0xff; b++) winansi[b] = b;
Object.entries({
  0x80: 0x20ac, 0x82: 0x201a, 0x83: 0x0192, 0x84: 0x201e, 0x85: 0x2026,
  0x86: 0x2020, 0x87: 0x2021, 0x88: 0x02c6, 0x89: 0x2030, 0x8a: 0x0160,
  0x8b: 0x2039, 0x8c: 0x0152, 0x8e: 0x017d, 0x91: 0x2018, 0x92: 0x2019,
  0x93: 0x201c, 0x94: 0x201d, 0x95: 0x2022, 0x96: 0x2013, 0x97: 0x2014,
  0x98: 0x02dc, 0x99: 0x2122, 0x9a: 0x0161, 0x9b: 0x203a, 0x9c: 0x0153,
  0x9e: 0x017e, 0x9f: 0x0178,
}).forEach(([b, u]) => { winansi[+b] = u; });

const out = [
  '//! Adobe AFM metrics for the standard Helvetica faces.',
  '//!',
  '//! Generated from the same AFM tables the reference renderer uses, so',
  '//! measurement — and therefore line breaking — matches it exactly. Widths are',
  '//! in 1/1000 em, indexed by WinAnsi byte; kern pairs are sorted for binary',
  '//! search.',
  '//!',
  '//! Regenerate with `scripts/gen-metrics.mjs`; do not edit by hand.',
  '',
];

for (const [fontName, ident] of [['Helvetica', 'HELVETICA'], ['Helvetica-Bold', 'HELVETICA_BOLD']]) {
  doc.font(fontName);
  const font = doc._font;
  const width = (s) => font.widthOfString(s, 1000);

  const widths = new Array(256).fill(0);
  const chars = [];
  for (let b = 0; b < 256; b++) {
    if (winansi[b] === null) continue;
    const ch = String.fromCodePoint(winansi[b]);
    widths[b] = Math.round(width(ch));
    chars.push([b, ch]);
  }

  const kern = [];
  for (const [b1, c1] of chars) {
    for (const [b2, c2] of chars) {
      const k = Math.round(width(c1 + c2) - widths[b1] - widths[b2]);
      if (k !== 0) kern.push([b1, b2, k]);
    }
  }
  kern.sort((a, b) => (a[0] - b[0]) || (a[1] - b[1]));

  out.push(`/// Advance widths for ${fontName}, in 1/1000 em, indexed by WinAnsi byte.`);
  out.push(`pub static ${ident}_WIDTHS: [u16; 256] = [`);
  for (let i = 0; i < 256; i += 16) out.push('    ' + widths.slice(i, i + 16).join(', ') + ',');
  out.push('];');
  out.push('');
  out.push(`/// Non-zero kern pairs for ${fontName}: (left, right, adjustment).`);
  out.push(`pub static ${ident}_KERN: &[(u8, u8, i16)] = &[`);
  for (const [a, b, k] of kern) out.push(`    (${a}, ${b}, ${k}),`);
  out.push('];');
  out.push('');
  console.error(`${fontName}: ${kern.length} kern pairs`);
}

const target = process.argv[2];
if (!target) throw new Error('usage: gen-metrics.mjs <output.rs>');
fs.writeFileSync(target, out.join('\n'));
console.error(`wrote ${target}`);

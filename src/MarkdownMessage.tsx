import React from "react";
import "./markdown.css";

type Segment =
  | { type: "code"; lang: string; content: string }
  | { type: "text"; lines: string[] };

export function MarkdownMessage({ content }: { content: string }) {
  const segments = splitFencedCode(content);
  return (
    <div className="markdownMessage">
      {segments.map((segment, index) =>
        segment.type === "code" ? (
          <CodeBlock key={index} lang={segment.lang} content={segment.content} />
        ) : (
          <TextBlock key={index} lines={segment.lines} />
        )
      )}
    </div>
  );
}

function splitFencedCode(content: string): Segment[] {
  const lines = content.replace(/\r\n/g, "\n").split("\n");
  const segments: Segment[] = [];
  let textLines: string[] = [];
  let codeLines: string[] = [];
  let inCode = false;
  let lang = "";

  for (const line of lines) {
    const fence = line.match(/^```\s*([^`]*)\s*$/);
    if (fence) {
      if (inCode) {
        segments.push({ type: "text", lines: textLines });
        textLines = [];
        segments.push({ type: "code", lang, content: codeLines.join("\n") });
        codeLines = [];
        lang = "";
        inCode = false;
      } else {
        segments.push({ type: "text", lines: textLines });
        textLines = [];
        lang = fence[1]?.trim() || "text";
        inCode = true;
      }
      continue;
    }
    if (inCode) codeLines.push(line);
    else textLines.push(line);
  }

  if (inCode) segments.push({ type: "code", lang, content: codeLines.join("\n") });
  else segments.push({ type: "text", lines: textLines });
  return segments.filter((segment) => segment.type === "code" || segment.lines.some((line) => line.trim()));
}

function TextBlock({ lines }: { lines: string[] }) {
  const nodes: React.ReactNode[] = [];
  let paragraph: string[] = [];
  let list: string[] = [];
  let orderedList: string[] = [];
  let table: string[] = [];

  function flushParagraph() {
    if (!paragraph.length) return;
    nodes.push(<p key={`p-${nodes.length}`}>{renderInline(paragraph.join(" "))}</p>);
    paragraph = [];
  }
  function flushList() {
    if (list.length) nodes.push(<ul key={`ul-${nodes.length}`}>{list.map((item, index) => <li key={index}>{renderInline(item)}</li>)}</ul>);
    list = [];
    if (orderedList.length) nodes.push(<ol key={`ol-${nodes.length}`}>{orderedList.map((item, index) => <li key={index}>{renderInline(item)}</li>)}</ol>);
    orderedList = [];
  }
  function flushTable() {
    if (table.length >= 2 && isDividerRow(table[1])) {
      nodes.push(<MarkdownTable key={`table-${nodes.length}`} rows={table} />);
    } else if (table.length) {
      paragraph.push(...table);
    }
    table = [];
  }
  function flushAll() {
    flushTable();
    flushList();
    flushParagraph();
  }

  for (const line of lines) {
    const trimmed = line.trim();
    if (!trimmed) {
      flushAll();
      continue;
    }
    if (trimmed.includes("|") && !trimmed.startsWith("#")) {
      flushList();
      flushParagraph();
      table.push(trimmed);
      continue;
    }
    flushTable();

    const heading = trimmed.match(/^(#{1,4})\s+(.+)$/);
    if (heading) {
      flushList();
      flushParagraph();
      const level = heading[1].length;
      const children = renderInline(heading[2]);
      nodes.push(React.createElement(`h${level + 1}`, { key: `h-${nodes.length}` }, children));
      continue;
    }

    const quote = trimmed.match(/^>\s+(.+)$/);
    if (quote) {
      flushList();
      flushParagraph();
      nodes.push(<blockquote key={`q-${nodes.length}`}>{renderInline(quote[1])}</blockquote>);
      continue;
    }

    const unordered = trimmed.match(/^[-*]\s+(.+)$/);
    if (unordered) {
      flushParagraph();
      orderedList = [];
      list.push(unordered[1]);
      continue;
    }

    const ordered = trimmed.match(/^\d+[.)]\s+(.+)$/);
    if (ordered) {
      flushParagraph();
      list = [];
      orderedList.push(ordered[1]);
      continue;
    }

    flushList();
    paragraph.push(trimmed);
  }
  flushAll();
  return <>{nodes}</>;
}

function MarkdownTable({ rows }: { rows: string[] }) {
  const cells = rows.map((row) => row.split("|").map((cell) => cell.trim()).filter(Boolean));
  const header = cells[0] || [];
  const body = cells.slice(2);
  return (
    <div className="markdownTableWrap">
      <table>
        <thead><tr>{header.map((cell, index) => <th key={index}>{renderInline(cell)}</th>)}</tr></thead>
        <tbody>{body.map((row, rowIndex) => <tr key={rowIndex}>{row.map((cell, index) => <td key={index}>{renderInline(cell)}</td>)}</tr>)}</tbody>
      </table>
    </div>
  );
}

function CodeBlock({ lang, content }: { lang: string; content: string }) {
  const normalizedLang = lang.toLowerCase();
  const label = normalizedLang.includes("plantuml")
    ? "PlantUML source / offline preview disabled"
    : normalizedLang.includes("mermaid")
      ? "Mermaid source / offline preview disabled"
      : normalizedLang || "code";
  return (
    <figure className="markdownCodeBlock">
      <figcaption>{label}</figcaption>
      <pre><code>{content}</code></pre>
    </figure>
  );
}

function isDividerRow(row: string) {
  return /^\|?\s*:?-{3,}:?\s*(\|\s*:?-{3,}:?\s*)+\|?$/.test(row);
}

function renderInline(text: string): React.ReactNode[] {
  const nodes: React.ReactNode[] = [];
  const pattern = /(`[^`]+`|\*\*[^*]+\*\*|__[^_]+__|\*[^*]+\*|_[^_]+_)/g;
  let cursor = 0;
  for (const match of text.matchAll(pattern)) {
    if (match.index === undefined) continue;
    if (match.index > cursor) nodes.push(text.slice(cursor, match.index));
    const token = match[0];
    if (token.startsWith("`")) nodes.push(<code key={nodes.length}>{token.slice(1, -1)}</code>);
    else if (token.startsWith("**") || token.startsWith("__")) nodes.push(<strong key={nodes.length}>{token.slice(2, -2)}</strong>);
    else nodes.push(<em key={nodes.length}>{token.slice(1, -1)}</em>);
    cursor = match.index + token.length;
  }
  if (cursor < text.length) nodes.push(text.slice(cursor));
  return nodes;
}

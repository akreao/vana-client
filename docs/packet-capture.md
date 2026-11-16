# Packet capture

A development aid for comparing servers: the client can record the map server's traffic in both directions.

## Turning it on

- `"packet_capture": true` in `config.json`, or
- `VANA_CAPTURE=1` in the environment (`VANA_CAPTURE=/some/folder` writes there instead).

Each client run writes one file, `<data dir>/vana-client/captures/<unix seconds>-<process id>.jsonl` (on Linux
`~/.local/share/vana-client/captures`), across every zone and character of the run. Files are created readable by
your user only. `vana-client login --zone-in` captures too.

## What is in a capture

One JSON object a line:

- a sub-packet, with its 4-byte header, as it went out or came in:
  `{"t":1763300000123,"dir":"in","zone":230,"id":143,"hex":"8f2c…"}`
- a datagram: its size in bytes, header sequence and ack, and the coder's form (`huffman`, `plain` for the copied
  form behind a byte other than 1, `raw` for the unencrypted first 0x00A):
  `{"t":…,"dir":"in","zone":230,"dgram":312,"seq":41,"ack":17,"form":"huffman"}`

`t` is Unix time in milliseconds; `zone` is the one the last s2c 0x00A named. Each sub-packet is written once: when
it first goes out, and when it is first accepted coming in. A resend (ours of an unacknowledged sub-packet, or the
server's of a datagram) appears only as a datagram line. An incoming datagram's line comes after the sub-packets it carried,
so a zone-in's datagram is counted in the zone it enters.

Only the map session is recorded. The lobby, PlayOnline sign-in and patch connections never are, since they carry
the password and account keys. The map login (c2s 0x00A) is written with its account name and ticket zeroed. Chat
and tells are recorded as they arrive, so treat a capture as private and cut it down before sharing it.

## Reading one

```
vana-client capture FILE [--id 08f,0c5] [--dir in|out] [--zone N] [--find 1e09] [--hex] [--save OUT]
```

It prints the kept sub-packets with their time from the start, direction, zone, id and common name and size (`--hex`
adds the bytes), then a count of each id each way and of each datagram form each way (datagram counts follow `--dir` and `--zone`). `--find` keeps sub-packets
that contain the given bytes anywhere. `--save` writes the kept lines to a new file, so you can share only the
packets that matter.

## The Packets window

With `"dev_tools": true` in `config.json` and the modern UI, `/packets` opens a window over the game:

- **Live** follows the map session as it runs (the window keeps the last 200,000 records); **Go live** returns to it
  after a file was opened. **Record to file** writes a capture as `packet_capture` does, for as long as it is lit.
- **Open capture** lists the captures folder, newest first; **Save shown** writes the rows the filters keep to a new
  file there (a name typed beside it, or a stamped one).
- Filters: ids (hex, `08f, 0c5`), direction, zone, and bytes (`1e 09`, or `"text"` in quotes). **Datagrams** shows
  the datagram rows (only while no id or bytes are filtered on); **Follow** keeps the newest row in view.
- Picking a row shows its header, the fields the client reads from that id (offsets from its own parsers and the
  decompile notes), and a hex dump with **Copy hex**.

`vana-client modern-shot --packets --out shot.png` draws the window with sample records.

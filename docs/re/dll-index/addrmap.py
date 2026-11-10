"""Translate FFXiMain.dll addresses between two builds through a map written by remap.py (maps/<older build>.tsv:
that build's addresses -> the build names.tsv is for)."""
import bisect, os


class Map:
    def __init__(self, path):
        self.since = None                                        # notes dated before this are about the older build
        self.old = os.path.basename(path)[:-4]
        rows = []
        for l in open(path):
            if l.startswith('#'):
                if l.startswith('# since '):
                    self.since = l.split()[2]
                continue
            k, a0, a1, d = l.rstrip('\n').split('\t')
            rows.append((int(a0, 16), int(a1, 16), int(d), k))
        self.f = sorted(rows)
        self.b = sorted((a0 + d, a1 + d, -d, k) for a0, a1, d, k in rows)

    @staticmethod
    def _go(rows, a):
        i = bisect.bisect_right(rows, (a, 1 << 40, 0, '')) - 1
        if i >= 0 and rows[i][0] <= a < rows[i][1]:
            return a + rows[i][2]
        return None

    def fwd(self, a):
        """Older build's address -> current build's (None: not mapped, e.g. inside a changed function)."""
        return self._go(self.f, a)

    def back(self, a):
        return self._go(self.b, a)


def load(here):
    """The newest map in <here>/maps, or None."""
    d = os.path.join(here, 'maps')
    fs = sorted(f for f in os.listdir(d) if f.endswith('.tsv')) if os.path.isdir(d) else []
    return Map(os.path.join(d, fs[-1])) if fs else None

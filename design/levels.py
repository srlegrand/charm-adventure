# Builds the glasshouse and cannery levels from reusable stretches (run, thorns, steps, pit, tower, corridor, arena).
# The Rootway is hand-written; these two are assembled so their jumps use the same proven spacings.
H = 1200
class Build:
    def __init__(self): self.x = 60; self.S = []; self.Z = []; self.E = []; self.pits = []
    def foe(self, kind, x, y): self.E.append((kind, x, y))
    def run(self, w, foes):
        x = self.x
        self.S += [(x + 160, 220, 140, 22), (x + 400, 330, 140, 22)]
        for i, k in enumerate(foes): self.foe(k, x + 260 + i * (w - 320) / max(len(foes), 1), 124)
        self.foe('cherry', x + 470, 360); self.x += w
    def thorns(self, n):
        x = self.x + 100
        for i in range(n):
            self.Z.append((x, 100, 220, 26)); self.S += [(x - 20, 230, 90, 22), (x + 160, 300, 90, 22)]
            self.foe('cherry' if i % 2 == 0 else 'hopper', x + 200, 330); x += 360
        self.x = x
    def steps(self):
        x = self.x
        for dx, w, h in [(0, 160, 110), (160, 160, 220), (320, 220, 330), (540, 160, 220), (700, 160, 110)]: self.S.append((x + dx, 100, w, h))
        self.foe('hopper', x + 430, 460); self.foe('cherry', x + 240, 350); self.x += 860
    def pit(self, w=240):
        x = self.x; self.pits.append((x, w)); self.S += [(x, 0, w, 20), (x + w / 2 - 30, 150, 60, 24)]; self.Z.append((x, 20, w, 30)); self.x += w
    def tower(self):
        x = self.x + 200
        self.S += [(x, 330, 140, 600), (x + 440, 100, 140, 700)]
        for i, y in enumerate(range(220, 800, 120)): self.S.append((x + 140 if i % 2 == 0 else x + 330, y, 110, 22))
        self.S += [(x + 580, 100, 300, 560), (x + 880, 100, 300, 330), (x + 1180, 100, 200, 110)]
        self.foe('cherry', x + 290, 124); self.foe('cherry', x + 700, 690); self.foe('hopper', x + 1000, 460); self.x = x + 1380
    def corridor(self, n, hoppers=2):
        x = self.x; w = 260 + n * 100
        self.S.append((x, 430, w, H - 60 - 430))
        for i in range(n): self.foe('cherry', x + 160 + i * 100, 120)
        for i in range(hoppers): self.foe('hopper', x + 300 + i * (w - 400) / max(hoppers - 1, 1), 124)
        self.x += w
    def arena(self, beef):
        x = self.x
        self.S += [(x + 180, 300, 160, 24), (x + 460, 440, 220, 24), (x + 780, 300, 160, 24), (x + 320, 620, 140, 24), (x + 680, 620, 140, 24)]
        for i in range(beef): self.foe('beefsteak', x + 260 + i * 560 / max(beef - 1, 1), 130)
        self.foe('cherry', x + 560, 470); self.foe('hopper', x + 390, 650); self.foe('hopper', x + 750, 650); self.x += 1100
    def write(self, path, name, theme, to, note):
        W = self.x + 60; S = [(0, 0, 60, H), (W - 60, 0, 60, H), (0, H - 60, W, 60)]; a = 0
        for px, pw in sorted(self.pits) + [(W, 0)]:
            S.append((a, 0, px - a, 100)); a = px + pw
        f = lambda t: '(' + ', '.join(f'{float(v):.1f}' for v in t) + ')'
        exits = f'        (rect: ({W-240:.1f}, 100.0, 160.0, 320.0), to: "{to}"),\n' if to else ''
        out = [f'// {name}. Written by design/levels.py: change that and run it again, or edit this by hand.', f'// {note}',
               '(', f'    name: "{name}",', f'    theme: "{theme}",', f'    bounds: (0.0, 0.0, {W:.1f}, {H:.1f}),', '    spawns: [(160.0, 160.0), (220.0, 160.0)],',
               '    exits: [\n' + exits + '    ],', '    solids: [', *[f'        {f(s)},' for s in S + self.S], '    ],', '    hazards: [', *[f'        {f(z)},' for z in self.Z], '    ],',
               '    enemies: [', *[f'        (kind: "{k}", x: {x:.1f}, y: {y:.1f}),' for k, x, y in self.E if x >= 700 or y > 200], '    ],', ')']
        open(path, 'w').write('\n'.join(out) + '\n'); print(path, W)
g = Build(); g.run(700, ['cherry', 'cherry']); g.thorns(2); g.steps(); g.pit(); g.run(800, ['hopper', 'cherry', 'cherry']); g.tower(); g.corridor(8); g.arena(1)
g.write('../assets/levels/glasshouse.ron', 'The Glasshouse', 'glasshouse', 'cannery', 'Route: run, thorn beds, steps, pit, run, tower climb, corridor, arena, door to the Cannery.')
c = Build(); c.run(600, ['hopper', 'cherry']); c.pit(260); c.thorns(3); c.tower(); c.steps(); c.corridor(12, 3); c.pit(); c.run(700, ['hopper', 'hopper', 'cherry']); c.arena(2)
c.write('../assets/levels/cannery.ron', 'The Cannery', 'cannery', '', 'Route: run, pit, thorn beds, tower climb, steps, long corridor, pit, run, arena. No door yet: the story ends here.')

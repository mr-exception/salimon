# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: edges.spec.ts >> ship rays reach every edge/corner; no object guides or arrowheads; zero, differing, overlapping and opposite vectors
- Location: tests/edges.spec.ts:17:1

# Error details

```
Test timeout of 30000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=f10e3]:
  - region "Navigation radar" [ref=f10e4]:
    - img "Ship-centered three-dimensional radar" [ref=f10e7]
    - generic: SALIMON 3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE
    - img "Ship heading and velocity vector lines"
    - generic: "HEADING VELOCITY · dashed · Scanner 100,000 light-seconds · pulse #1"
  - complementary [ref=f10e8]:
    - heading "NAVIGATION" [level=1] [ref=f10e9]
    - paragraph [ref=f10e10]: Navigation-only prototype · unlimited energy
    - status [ref=f10e11]: "Scanner pulse 1: 8 temporary contacts"
    - generic [ref=f10e12]:
      - generic [ref=f10e13]: Total ship mass
      - strong [ref=f10e14]: 100,000 kg
    - generic [ref=f10e15]:
      - generic [ref=f10e16]: Relative position
      - strong [ref=f10e17]: 0.0 / 3.0 / 0.0 m (cell local)
    - generic [ref=f10e18]:
      - generic [ref=f10e19]: Speed (sample inertial frame)
      - strong [ref=f10e20]: 1.0 m/s
    - generic [ref=f10e21]:
      - generic [ref=f10e22]: Yaw / pitch
      - strong [ref=f10e23]: "-90.0° / 0.0°"
    - generic [ref=f10e24]:
      - generic [ref=f10e25]: Alignment
      - strong [ref=f10e26]: Manual
    - generic [ref=f10e27]:
      - generic [ref=f10e28]: Flight status
      - strong [ref=f10e29]: flying
    - status "Navigation warnings" [ref=f10e30]:
      - paragraph [ref=f10e31]: No navigation warnings
    - separator [ref=f10e32]
    - heading "Contact" [level=2] [ref=f10e33]
    - status [ref=f10e34]: No selected target
    - generic [ref=f10e35]: Find scanned contacts
    - searchbox "Find scanned contacts" [ref=f10e36]
    - generic [ref=f10e37]: Scanned contacts (8)
    - combobox "Scanned contacts (8)" [ref=f10e38]:
      - option "Choose a contact" [selected]
      - option "bottom"
      - option "bottom-left"
      - option "bottom-right"
      - option "left"
      - option "right"
      - option "top"
      - option "top-left"
      - option "top-right"
    - paragraph [ref=f10e39]: Select a visible contact or use the scanned-contact list.
    - separator [ref=f10e40]
    - heading "Camera and orientation" [level=2] [ref=f10e41]
    - paragraph [ref=f10e42]: WASD pans the view · wheel zooms · arrow keys turn the ship.
    - generic [ref=f10e43]:
      - button "Help" [ref=f10e44] [cursor=pointer]
      - button "Reset camera" [ref=f10e45] [cursor=pointer]
    - separator [ref=f10e46]
    - heading "Throttle 0%" [level=2] [ref=f10e47]:
      - generic [ref=f10e48]: Throttle
      - strong [ref=f10e49]: 0%
    - slider "Throttle" [ref=f10e50]: "0"
    - button "Zero throttle (Space)" [ref=f10e51] [cursor=pointer]
    - paragraph [ref=f10e52]: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.
```

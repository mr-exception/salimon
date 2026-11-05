# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: cockpit.spec.ts >> COCKPIT-1 radar round trip, reserved seat, blackout and persistent simulation
- Location: tests/cockpit.spec.ts:5:1

# Error details

```
Test timeout of 180000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=e2]:
  - generic [ref=e3]:
    - region "Navigation radar" [ref=e4]:
      - img "Ship-centered three-dimensional radar" [ref=e7]
      - generic: SALIMON 3D HOLOGRAPHIC RADAR / SAMPLE UNIVERSE
      - img "Ship heading and velocity vector lines"
      - button "◉ 2 contacts" [ref=e8] [cursor=pointer]
      - generic: "HEADING STATIONARY · Scanner 100,000 light-seconds · pulse #2"
    - complementary [ref=e9]:
      - heading "NAVIGATION" [level=1] [ref=e10]
      - paragraph [ref=e11]: Navigation-only prototype · unlimited energy
      - status [ref=e12]: "Scanner pulse 2: 5 temporary contacts"
      - generic [ref=e13]:
        - generic [ref=e14]: Total ship mass
        - strong [ref=e15]: 100,000 kg
      - generic [ref=e16]:
        - generic [ref=e17]: Relative position
        - strong [ref=e18]: 0.0 / 0.0 / 0.0 m (cell local)
      - generic [ref=e19]:
        - generic [ref=e20]: Speed (sample inertial frame)
        - strong [ref=e21]: 0.0 m/s
      - generic [ref=e22]:
        - generic [ref=e23]: Yaw / pitch
        - strong [ref=e24]: 0.0° / 0.0°
      - generic [ref=e25]:
        - generic [ref=e26]: Alignment
        - strong [ref=e27]: Manual
      - generic [ref=e28]:
        - generic [ref=e29]: Flight status
        - strong [ref=e30]: flying
      - status "Navigation warnings" [ref=e31]:
        - paragraph [ref=e32]: No navigation warnings
      - separator [ref=e33]
      - heading "Contact" [level=2] [ref=e34]
      - status [ref=e35]: No selected target
      - generic [ref=e36]: Find scanned contacts
      - searchbox "Find scanned contacts" [ref=e37]
      - generic [ref=e38]: Scanned contacts (5)
      - combobox "Scanned contacts (5)" [ref=e39]:
        - option "Choose a contact" [selected]
        - option "Caligo"
        - option "Nacre"
        - option "Uncharted asteroid"
        - option "Vesper"
        - option "Vesper Moon"
      - paragraph [ref=e40]: Select a visible contact or use the scanned-contact list.
      - separator [ref=e41]
      - heading "Camera and orientation" [level=2] [ref=e42]
      - paragraph [ref=e43]: WASD pans the view · wheel zooms · arrow keys turn the ship.
      - generic [ref=e44]:
        - button "Help" [ref=e45] [cursor=pointer]
        - button "Reset camera" [ref=e46] [cursor=pointer]
      - separator [ref=e47]
      - heading "Throttle 0%" [level=2] [ref=e48]:
        - generic [ref=e49]: Throttle
        - strong [ref=e50]: 0%
      - slider "Throttle" [ref=e51]: "0"
      - button "Zero throttle (Space)" [ref=e52] [cursor=pointer]
      - paragraph [ref=e53]: Illustrative universe · unlimited energy. Impacts at 100 m/s or faster end the run. Lower-speed contact settles the ship.
  - button "Exit navigation · Stand up" [ref=e54] [cursor=pointer]
  - button "Test cockpit power loss" [ref=e56] [cursor=pointer]
```

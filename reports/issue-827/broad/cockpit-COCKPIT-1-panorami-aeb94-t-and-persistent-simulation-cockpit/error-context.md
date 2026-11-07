# Instructions

- Following Playwright test failed.
- Explain why, be concise, respect Playwright best practices.
- Provide a snippet of code with the fix, if possible.

# Test info

- Name: cockpit.spec.ts >> COCKPIT-1 panoramic round trip, reserved seat, blackout and persistent simulation
- Location: tests/cockpit.spec.ts:10:1

# Error details

```
Test timeout of 360000ms exceeded.
```

# Page snapshot

```yaml
- generic [ref=e3]:
  - complementary "Survival condition" [ref=e4]:
    - generic [ref=e5]:
      - generic [ref=e6]:
        - generic [ref=e7]: "Health: 200 / 200 HP"
        - meter "Health" [ref=e8]
      - generic [ref=e9]:
        - generic [ref=e10]: "Hunger: 99 / 100"
        - meter "Hunger" [ref=e11]
      - generic [ref=e12]:
        - generic [ref=e13]: "Fatigue: 100 / 100"
        - meter "Fatigue" [ref=e14]
    - button "Pause [P]" [ref=e15] [cursor=pointer]
    - button "Help" [ref=e17] [cursor=pointer]
  - region "Panoramic navigation" [ref=e19]:
    - generic "Panoramic universe" [ref=e22]
    - generic [aria-hidden]: +
    - generic:
      - strong: SALIMON · OBSERVATION
      - status: Scanner pulse 4 · 5 contacts
      - generic: grounded · Vesper · 0.0 m/s · sample inertial frame
    - status "Navigation warnings":
      - paragraph: safe-contact · impact 0.0 m/s
    - generic:
      - button "Sensor contact Vesper Moon" [ref=e23] [cursor=pointer]: +
      - region "Detected contacts" [ref=e24]:
        - button "Contacts (C) · 5" [expanded] [ref=e25] [cursor=pointer]
        - generic [ref=e26]:
          - generic [ref=e27]:
            - text: Search contacts
            - textbox "Search contacts" [ref=e28]
          - list [ref=e29]:
            - listitem [ref=e30]:
              - button "Caligo" [pressed] [ref=e31] [cursor=pointer]
            - listitem [ref=e32]:
              - button "Nacre" [ref=e33] [cursor=pointer]
            - listitem [ref=e34]:
              - button "Uncharted asteroid" [ref=e35] [cursor=pointer]
            - listitem [ref=e36]:
              - button "Vesper" [ref=e37] [cursor=pointer]
            - listitem [ref=e38]:
              - button "Vesper Moon" [ref=e39] [cursor=pointer]
          - button "Previous" [disabled] [ref=e40]
          - generic [ref=e41]: 1/1
          - button "Next" [disabled] [ref=e42]
        - generic [ref=e43]:
          - status [ref=e44]: "Selected: Caligo"
          - button "Look at" [active] [ref=e45] [cursor=pointer]
          - button "Set direction" [disabled] [ref=e46]
    - generic:
      - figure "3D orientation compass":
        - 'img "Camera cyan at center. Heading yellow: front, right -14.6, up -8.3, depth 0.81. Velocity green: Stationary. Solid front, hollow rear."':
          - generic: UP
        - generic:
          - text: Cyan view · Yellow heading
          - emphasis: Stationary
          - text: · solid front / hollow rear
    - generic [ref=e47]:
      - generic [ref=e48]:
        - text: Throttle 0%
        - slider "Throttle" [disabled] [ref=e49]: "0"
      - button "Zero throttle (Space)" [disabled] [ref=e50]
      - button "Reset view" [ref=e51] [cursor=pointer]
      - button "Telemetry" [ref=e52] [cursor=pointer]
      - button "Help" [ref=e53] [cursor=pointer]
      - button "Exit cockpit" [ref=e54] [cursor=pointer]
  - button "Test cockpit power loss" [ref=e56] [cursor=pointer]
```
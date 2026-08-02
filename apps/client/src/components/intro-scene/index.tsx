import { useMemo, useState } from 'react';
import style from './style.module.css';

function getIntroSlides(year: number) {
  return [
    {
      marker: 'Transmission 01',
      text: [
        `It is ${year}. From the dark beyond every charted system, an unknown presence speaks to Earth.`,
        'Its first message is brief, impossible, and repeated across every listening band.',
        '"Reach the Absenat, where everything starts."',
      ],
    },
    {
      marker: 'Transmission 02',
      text: [
        'With the signal came an object no human instrument could explain.',
        'A battery-shaped core, silent and weightless beyond reason, able to hold and release energy at a scale that broke our physics.',
        'Humanity gave it the only name that made sense: the Energy Core.',
      ],
    },
    {
      marker: 'Transmission 03',
      text: [
        'Every twelve hours, another Core appears on Earth.',
        'At first, we used them to end the energy crisis. Then we built around them.',
        'Engines. Shields. Fabricators. Ships that could survive the distances between worlds.',
      ],
    },
    {
      marker: 'Launch Protocol',
      text: [
        'Now one vessel is ready, and its command is yours.',
        'Absenat remains unknown, but Mars carries the first scar in the pattern: an anomaly where no anomaly should be.',
        'Alien, machine, warning, or invitation, the only path begins there.',
      ],
    },
    {
      marker: 'Departure',
      text: [
        'The Core is awake.',
        'Earth is below you. Mars waits ahead.',
        'Reach the anomaly. Find the road to Absenat.',
      ],
    },
  ];
}

type Props = {
  onComplete: () => void;
};

export default function IntroScene({ onComplete }: Props) {
  const year = useMemo(() => new Date().getFullYear(), []);
  const slides = useMemo(() => getIntroSlides(year), [year]);
  const [activeSlideIndex, setActiveSlideIndex] = useState(0);
  const activeSlide = slides[activeSlideIndex];
  const isFinalSlide = activeSlideIndex === slides.length - 1;

  const advance = () => {
    if (isFinalSlide) {
      onComplete();
      return;
    }

    setActiveSlideIndex((current) => current + 1);
  };

  return (
    <section className={style.intro} aria-live="polite">
      <div className={style.starfield} aria-hidden="true" />
      <div className={style.vignette} aria-hidden="true" />
      <article className={style.slide} key={activeSlideIndex}>
        <p className={style.marker}>{activeSlide.marker}</p>
        <div className={style.crawl}>
          {activeSlide.text.map((line) => (
            <p key={line}>{line}</p>
          ))}
        </div>
      </article>
      <footer className={style.controls}>
        <div
          className={style.progress}
          aria-label={`Intro ${activeSlideIndex + 1} of ${slides.length}`}
        >
          {slides.map((slide, index) => (
            <span
              key={slide.marker}
              data-active={index === activeSlideIndex}
            />
          ))}
        </div>
        <button type="button" onClick={advance}>
          {isFinalSlide ? 'Begin' : 'Continue'}
        </button>
      </footer>
    </section>
  );
}

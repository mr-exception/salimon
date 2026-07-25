import type { Contact } from '../types';
import style from '../style.module.css';

type Props = {
  contact?: Contact;
};

export function ContactHeading({ contact }: Props) {
  const initials = contact?.name
    .split(/\s+/)
    .filter(Boolean)
    .slice(0, 2)
    .map((part) => part[0])
    .join('');

  return (
    <div className={style.contactHeading}>
      <span className={style.avatar} aria-hidden="true">
        {initials}
      </span>
      <span className={style.contactSummary}>
        <strong>{contact?.name ?? 'No contact selected'}</strong>
        <small>{contact?.organization}</small>
      </span>
    </div>
  );
}

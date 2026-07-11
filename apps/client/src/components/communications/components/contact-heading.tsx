import type { Contact } from '../types';
import style from '../style.module.css';

type Props = {
  contact?: Contact;
};

export function ContactHeading({ contact }: Props) {
  return (
    <div className={style.contactHeading}>
      <strong>{contact?.name ?? 'No contact selected'}</strong>
      <small>{contact?.organization}</small>
    </div>
  );
}

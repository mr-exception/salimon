import type { Contact } from '../types';
import style from '../style.module.css';

type Props = {
  contacts: Contact[];
  selectedContactId?: string;
  unreadCountByContact: Map<string, number>;
  onSelectContact: (contactId: string) => void;
};

export function ContactsList({
  contacts,
  selectedContactId,
  unreadCountByContact,
  onSelectContact,
}: Props) {
  return (
    <nav className={style.contacts} aria-label="Known contacts">
      {contacts.map((contact) => {
        const unreadCount = unreadCountByContact.get(contact.id) ?? 0;
        const initials = contact.name
          .split(/\s+/)
          .filter(Boolean)
          .slice(0, 2)
          .map((part) => part[0])
          .join('');
        return (
          <button
            type="button"
            key={contact.id}
            data-active={contact.id === selectedContactId}
            data-unread={unreadCount > 0}
            onClick={() => onSelectContact(contact.id)}
          >
            <span className={style.avatar} aria-hidden="true">
              {initials}
            </span>
            <span className={style.contactSummary}>
              <span>{contact.name}</span>
              <small>{contact.organization}</small>
            </span>
            {unreadCount > 0 && (
              <strong aria-label={`${unreadCount} unread`}>
                {unreadCount}
              </strong>
            )}
          </button>
        );
      })}
    </nav>
  );
}

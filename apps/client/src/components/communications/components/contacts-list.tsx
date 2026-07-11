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
        return (
          <button
            type="button"
            key={contact.id}
            data-active={contact.id === selectedContactId}
            data-unread={unreadCount > 0}
            onClick={() => onSelectContact(contact.id)}
          >
            <span>{contact.name}</span>
            <small>{contact.organization}</small>
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

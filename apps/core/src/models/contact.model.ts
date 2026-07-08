import { DatabaseModel } from './database.model';

export type ContactDocument = {
  spaceshipSecurityCode: string;
  contactId: string;
  unlockedAt: Date;
  lastReadAt?: Date;
};

let indexesPromise: Promise<unknown> | undefined;

export class ContactModel {
  static async getCollection() {
    const collection = (
      await DatabaseModel.getDatabase()
    ).collection<ContactDocument>('contacts');
    indexesPromise ??= collection.createIndex(
      { spaceshipSecurityCode: 1, contactId: 1 },
      { unique: true },
    );
    await indexesPromise;
    return collection;
  }

  static async findBySpaceshipSecurityCode(spaceshipSecurityCode: string) {
    return (await ContactModel.getCollection())
      .find({ spaceshipSecurityCode })
      .toArray();
  }

  static async findBySpaceshipAndContact(
    spaceshipSecurityCode: string,
    contactId: string,
  ) {
    return (await ContactModel.getCollection()).findOne({
      spaceshipSecurityCode,
      contactId,
    });
  }

  static async upsertSpaceshipContact(contact: ContactDocument) {
    return (await ContactModel.getCollection()).updateOne(
      {
        spaceshipSecurityCode: contact.spaceshipSecurityCode,
        contactId: contact.contactId,
      },
      { $setOnInsert: contact },
      { upsert: true },
    );
  }
}

import { generateContactReply } from './generate-contact-reply';
import { generateContactReplyInBackground } from './generate-contact-reply-in-background';

export class ContactRepliesService {
  static generateContactReply = generateContactReply;
  static generateContactReplyInBackground = generateContactReplyInBackground;
}


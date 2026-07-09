import { SpaceshipService } from '../spaceship.service';

export function getSocketSecurityCode(url: URL) {
  return SpaceshipService.getSecurityCode({
    'x-spaceship-security-code':
      url.searchParams.get('shipSecret') ??
      url.searchParams.get('securityCode') ??
      url.searchParams.get('secret'),
  });
}

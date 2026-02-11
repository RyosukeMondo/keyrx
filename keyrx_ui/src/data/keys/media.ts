/**
 * Media key definitions (volume, play, brightness, etc.)
 */
import { KeyDefinition } from './types';

export const MEDIA: KeyDefinition[] = [
  {
    id: 'MediaPlayPause',
    label: 'Play/Pause',
    category: 'media',
    description: 'Media Play/Pause',
    aliases: ['KC_MPLY', 'VK_MEDIA_PLAY_PAUSE', 'KEY_PLAYPAUSE'],
  },
  {
    id: 'MediaStop',
    label: 'Stop',
    category: 'media',
    description: 'Media Stop',
    aliases: ['KC_MSTP', 'VK_MEDIA_STOP', 'KEY_STOPCD'],
  },
  {
    id: 'MediaPrev',
    label: 'Previous',
    category: 'media',
    description: 'Media Previous Track',
    aliases: ['KC_MPRV', 'VK_MEDIA_PREV_TRACK', 'KEY_PREVIOUSSONG'],
  },
  {
    id: 'MediaNext',
    label: 'Next',
    category: 'media',
    description: 'Media Next Track',
    aliases: ['KC_MNXT', 'VK_MEDIA_NEXT_TRACK', 'KEY_NEXTSONG'],
  },
  {
    id: 'VolumeUp',
    label: 'Vol+',
    category: 'media',
    description: 'Volume Up',
    aliases: ['KC_VOLU', 'VK_VOLUME_UP', 'KEY_VOLUMEUP'],
  },
  {
    id: 'VolumeDown',
    label: 'Vol-',
    category: 'media',
    description: 'Volume Down',
    aliases: ['KC_VOLD', 'VK_VOLUME_DOWN', 'KEY_VOLUMEDOWN'],
  },
  {
    id: 'VolumeMute',
    label: 'Mute',
    category: 'media',
    description: 'Volume Mute',
    aliases: ['KC_MUTE', 'VK_VOLUME_MUTE', 'KEY_MUTE'],
  },
  {
    id: 'BrightnessUp',
    label: 'Bright+',
    category: 'media',
    description: 'Brightness Up',
    aliases: ['KC_BRIU', 'KEY_BRIGHTNESSUP'],
  },
  {
    id: 'BrightnessDown',
    label: 'Bright-',
    category: 'media',
    description: 'Brightness Down',
    aliases: ['KC_BRID', 'KEY_BRIGHTNESSDOWN'],
  },
];

import type { UserActivity } from './types';

// 1x1 transparent GIF - used to force browser to release decoded bitmap memory
const BLANK_GIF = 'data:image/gif;base64,R0lGODlhAQABAIAAAAAAAP///yH5BAEAAAAALAAAAAABAAEAAAIBRAA7';

/**
 * Extended activity record that tracks when the user was last seen,
 * stored as a Date so the sort can compare against "within 30 seconds".
 */
const userActivityData: Record<string, UserActivity & { _lastActivity?: Date }> = {};

/**
 * Return the current set of active users (consumed by @-mention autocomplete).
 */
export function getActiveUsers(): Record<string, UserActivity> {
    return userActivityData;
}

/**
 * Add, update, or remove a single user's activity entry.
 *
 * - `activity` object  -> store and touch the sidebar element
 * - `activity === false` -> delete the entry and remove the DOM element
 * - Guest id `"0"` is silently ignored
 */
export function userActivity(id: string, activity: UserActivity | false): void {
    if (id === '0') return;

    const userEl = document.getElementById(`chat-activity-${id}`);

    if (activity !== false) {
        userActivityData[id] = activity;
        userActivityTouch(id);
    } else {
        delete userActivityData[id];

        if (userEl) {
            cleanupAvatarImage(userEl.querySelector('.avatar') as HTMLImageElement | null);
            userEl.remove();
        }
    }
}

/**
 * Remove every activity element and reset the data store.
 * Called when switching rooms so the sidebar is rebuilt from scratch.
 */
export function userActivityDelete(): void {
    const userEl = document.getElementById('chat-activity');
    if (!userEl) return;

    while (userEl.firstChild) {
        const child = userEl.firstChild as HTMLElement;
        cleanupAvatarImage(
            child.querySelector ? (child.querySelector('.avatar') as HTMLImageElement | null) : null,
        );
        userEl.removeChild(child);
    }

    // Clear the data object
    for (const key of Object.keys(userActivityData)) {
        delete userActivityData[key];
    }
}

/**
 * Update an existing activity element or create a brand-new one from the
 * `#tmp-chat-user` template.  Avatar images are only replaced when the
 * URL actually changes (tracked via `dataset.avatarUrl`) to avoid
 * unnecessary bitmap decoding.
 */
function userActivityTouch(id: string): void {
    const data = userActivityData[id];
    if (!data) return;

    let userEl = document.getElementById(`chat-activity-${id}`) as HTMLElement | null;
    data._lastActivity = new Date();

    if (userEl) {
        // Update the existing element's last_activity
        (userEl as any).last_activity = data._lastActivity;

        // Only update avatar if the URL has ACTUALLY changed.
        // Store original URL in dataset to avoid absolute vs relative comparison issues.
        const avEl = userEl.querySelector('.avatar') as HTMLImageElement | null;
        const newUrl = data.avatar_url || '';
        const currentUrl = userEl.dataset.avatarUrl || '';

        if (newUrl !== currentUrl) {
            userEl.dataset.avatarUrl = newUrl;

            if (newUrl && avEl) {
                // URL changed - replace the entire img element to ensure bitmap is released
                const newAvEl = document.createElement('img');
                newAvEl.classList.add('avatar');
                newAvEl.src = newUrl;
                newAvEl.alt = data.username;
                newAvEl.setAttribute('loading', 'lazy');
                newAvEl.setAttribute('decoding', 'async');

                // Clean up old element and replace
                cleanupAvatarImage(avEl);
                userEl.prepend(newAvEl);
            } else if (newUrl && !avEl) {
                // Avatar was previously absent, add one
                const addAvEl = document.createElement('img');
                addAvEl.classList.add('avatar');
                addAvEl.src = newUrl;
                addAvEl.alt = data.username;
                addAvEl.setAttribute('loading', 'lazy');
                addAvEl.setAttribute('decoding', 'async');
                userEl.prepend(addAvEl);
            } else if (!newUrl && avEl) {
                // User removed their avatar
                cleanupAvatarImage(avEl);
            }
        }
    } else {
        // Create a new element from the template
        const usersEl = document.getElementById('chat-activity');
        const template = document.getElementById('tmp-chat-user') as HTMLTemplateElement | null;
        if (!usersEl || !template) return;

        const newEl = (template.content.cloneNode(true) as DocumentFragment).children[0] as HTMLElement;

        newEl.id = `chat-activity-${id}`;
        newEl.dataset.username = data.username;
        newEl.dataset.avatarUrl = data.avatar_url || '';
        (newEl as any).last_activity = data._lastActivity;

        const avEl = newEl.querySelector('.avatar') as HTMLImageElement | null;
        if (data.avatar_url && avEl) {
            avEl.src = data.avatar_url;
            avEl.alt = data.username;
            avEl.setAttribute('loading', 'lazy');
            avEl.setAttribute('decoding', 'async');
        } else if (avEl) {
            avEl.remove();
        }

        const nameEl = newEl.querySelector('.user');
        if (nameEl) {
            nameEl.textContent = data.username;
        }

        usersEl.appendChild(newEl);
    }
}

/**
 * Sort activity elements: users active within the last 30 seconds are
 * placed first, then everything is ordered alphabetically by username.
 */
export function userActivitySort(): void {
    const usersEl = document.getElementById('chat-activity');
    if (!usersEl) return;

    const activityEls = usersEl.querySelectorAll('.activity');
    const time = new Date().getTime();

    const sorted = Array.from(activityEls).sort((a, b) => {
        const aEl = a as HTMLElement & { last_activity?: Date };
        const bEl = b as HTMLElement & { last_activity?: Date };

        const ar = aEl.last_activity ? (aEl.last_activity.getTime() - time) <= 30000 : false;
        const br = bEl.last_activity ? (bEl.last_activity.getTime() - time) <= 30000 : false;

        if (ar === br) {
            return (aEl.dataset.username || '').toLowerCase()
                .localeCompare((bEl.dataset.username || '').toLowerCase());
        } else if (ar && !br) {
            return -1;
        } else {
            return 1;
        }
    });

    sorted.forEach((e) => usersEl.appendChild(e));
}

/**
 * Release bitmap memory held by an avatar `<img>` element.
 *
 * Chrome doesn't reliably free decoded bitmaps when an element is simply
 * removed from the DOM.  Assigning a tiny transparent GIF first forces
 * the previous bitmap to be discarded.
 */
export function cleanupAvatarImage(avatarEl: HTMLImageElement | null): void {
    if (!avatarEl) return;

    // Force the browser to release the decoded bitmap by replacing with tiny image
    avatarEl.src = BLANK_GIF;

    // Remove other attributes
    avatarEl.removeAttribute('srcset');
    avatarEl.removeAttribute('loading');
    avatarEl.removeAttribute('decoding');
    avatarEl.removeAttribute('alt');

    // Now remove from DOM
    avatarEl.remove();
}

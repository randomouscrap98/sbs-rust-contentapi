use super::*;

/// Return whether a user can perform the given action on the given content. NOTE: this function 
/// expects the content to have permissions; it will SILENTLY FAIL if there are none! This may 
/// change in the future...
pub fn can_user_action(user: &User, action: &str, content: &Content) -> bool
{
    //Super users can do anything other than read
    if action != "R" && user.admin {
        return true;
    }

    //NOTE: this will FAIL SILENTLY if there are no permissions!!
    if let Some(ref permissions) = content.permissions 
    {
        let mut all_ids = user.groups.clone();
        all_ids.push(0);
        all_ids.push(user.id);

        for id in all_ids {
            if let Some(permVal) = permissions.get(&id.to_string()) {
                if permVal.contains(action) { 
                    return true;
                }
            }
        }
    }

    return false;
}

/// Return whether a user can edit a message or not. Note that if the message does not have the 
/// requisite fields, it SILENTLY FAILS and the result is false
pub fn can_user_edit_message(user: &User, post: &Message) -> bool
{
    return user.admin || Some(user.id) == post.createUserId;
}

/// Return whether a user can edit a message or not. Note that if the message does not have the 
/// requisite fields, it SILENTLY FAILS and the result is false
pub fn can_user_delete_message(user: &User, post: &Message) -> bool
{
    //Logic is same as editing, but may not be in the future? who knows...
    can_user_edit_message(user, post)
}
-- For each user, ensure a personal team exists (owner = user, name = "Personal") and set `members`
-- from legacy ACL: if user B lists user A in B.read, A's personal team includes B as guest;
-- if B lists A in B.write, B is content_maintainer (write wins when both apply). Owner is never in `members`.

FOR $owner IN (SELECT VALUE id FROM user) {
    LET $writer_ids = SELECT VALUE id FROM user WHERE $owner INSIDE write AND id != $owner;
    LET $reader_ids = SELECT VALUE id FROM user WHERE $owner INSIDE read AND id != $owner;
    LET $members_write = $writer_ids.map(|$uid| { user: $uid, role: 'content_maintainer' });
    LET $members_read = $reader_ids
        .filter(|$rid| !($rid INSIDE $writer_ids))
        .map(|$rid| { user: $rid, role: 'guest' });
    LET $members = array::concat($members_write, $members_read);
    LET $team_ids = SELECT VALUE id FROM team WHERE owner = $owner LIMIT 1;
    IF array::len($team_ids) > 0 {
        UPDATE $team_ids[0] SET members = $members;
    } ELSE {
        CREATE team SET name = 'Personal', owner = $owner, members = $members;
    };
};

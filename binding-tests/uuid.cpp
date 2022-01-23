#include <string.h>
#include "doctest.h"
#include "taskchampion.h"

TEST_CASE("creating UUIDs does not crash") {
    TCUuid u1 = tc_uuid_new_v4();
    TCUuid u2 = tc_uuid_nil();
}

TEST_CASE("converting UUIDs to string works") {
    TCUuid u2 = tc_uuid_nil();
    REQUIRE(TC_UUID_STRING_BYTES == 36);

    char u2str[TC_UUID_STRING_BYTES];
    tc_uuid_to_str(u2, u2str);
    CHECK(strncmp(u2str, "00000000-0000-0000-0000-000000000000", TC_UUID_STRING_BYTES) == 0);
}

TEST_CASE("converting UUIDs from string works") {
    TCUuid u;
    char ustr[TC_UUID_STRING_BYTES] = "fdc314b7-f938-4845-b8d1-95716e4eb762";
    CHECK(tc_uuid_from_str(ustr, &u));
    CHECK(u.bytes[0] == 0xfd);
    // .. if these two bytes are correct, then it probably worked :)
    CHECK(u.bytes[15] == 0x62);
}

TEST_CASE("converting invalid UUIDs from string fails as expected") {
    TCUuid u;
    char ustr[TC_UUID_STRING_BYTES] = "not-a-valid-uuid";
    CHECK(!tc_uuid_from_str(ustr, &u));
}

TEST_CASE("converting invalid UTF-8 UUIDs from string fails as expected") {
    TCUuid u;
    char ustr[TC_UUID_STRING_BYTES] = "\xf0\x28\x8c\xbc";
    CHECK(!tc_uuid_from_str(ustr, &u));
}
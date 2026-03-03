import pytest
import aiql


def test_sum_as_string():
    assert aiql.sum_as_string(1, 1) == "2"

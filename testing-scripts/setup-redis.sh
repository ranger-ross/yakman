export YAKMAN_ADAPTER=REDIS

redis-cli FLUSHALL
redis-cli SET CONFIG_MAN_CONFIGS '{"configs":[{"name":"Testing 1","description":"This is a test desc for config 1"},{"name":"Testing 2","description":"This is a test desc for config 2"},{"name":"Testing 3","description":"This is a test desc for config 3"}]}'
redis-cli SET CONFIG_MAN_LABELS '{"labels":[{"name":"My label","priority": 2,"description":"Here is my label","options":["option 1","option 2","option 3"]}]}'
redis-cli SET CONFIG_MAN_INSTANCE_META_100 '{"instances":[{"config_name":"100","instance":"100_ross_with_labels.json","labels":[{"label_type":"env","value":"dev"}]},{"config_name":"100","instance":"100_ross.json","labels":[]}]}'
redis-cli SET CONFIG_MAN_INSTANCE_100_ross_with_labels.json '{"data":"test data","labels":"some labels"}'
redis-cli SET CONFIG_MAN_INSTANCE_100_ross.json '{"data":"test data","labels":"No labels!"}'



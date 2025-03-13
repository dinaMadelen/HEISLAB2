package connectivity

import (
	"sync"
)

var (
	world_view_backup       [NR_OF_ELEVATORS]Worldview_package
	world_view_backup_mutex sync.Mutex
)

func Store_worldview(id int, worldview Worldview_package) {
	world_view_backup_mutex.Lock()
	defer world_view_backup_mutex.Unlock()
	world_view_backup[id] = worldview

}

func Get_worldview(id int) Worldview_package {
	world_view_backup_mutex.Lock()
	defer world_view_backup_mutex.Unlock()
	return world_view_backup[id]
}
